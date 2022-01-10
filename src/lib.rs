use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::iter;
use std::path::Path;
use tar::Archive;
use tar::Builder;

pub fn read_key_from_file(file_name: &str) -> age::x25519::Identity {
    let identify_file_entry = age::IdentityFile::from_file(file_name.to_string())
        .expect("Error reading key from file")
        .into_identities();
    // Bummed about this clone(), but can't figure out another way
    // right now
    match identify_file_entry[0].clone() {
        age::IdentityFileEntry::Native(i) => i,
        // _ => unreachable!(),
    }
}

fn encrypt_bytes(pubkey: age::x25519::Recipient, bytes_to_encrypt: &[u8]) -> Vec<u8> {
    let encryptor = age::Encryptor::with_recipients(vec![Box::new(pubkey)]);

    let mut encrypted_bytes = vec![];
    let mut writer = encryptor.wrap_output(&mut encrypted_bytes).unwrap();
    writer.write_all(bytes_to_encrypt).unwrap();
    writer.finish().unwrap();

    encrypted_bytes
}

fn decrypt_bytes(key: age::x25519::Identity, encrypted_bytes: Vec<u8>) -> Vec<u8> {
    let decryptor = match age::Decryptor::new(&encrypted_bytes[..]).unwrap() {
        age::Decryptor::Recipients(d) => d,
        _ => unreachable!(),
    };

    let mut decrypted = vec![];
    let mut reader = decryptor
        .decrypt(iter::once(&key as &dyn age::Identity))
        .unwrap();
    reader
        .read_to_end(&mut decrypted)
        .expect("Error decrypting file");

    decrypted
}

pub fn encrypt_file(key: age::x25519::Identity, target_file_name: &str) -> std::io::Result<()> {
    let target_file = fs::read(target_file_name)?;
    let encrypted_bytes = encrypt_bytes(key.to_public(), &target_file);

    let output_filename = Path::new(target_file_name)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    // add the .age extension
    write_file_to_system(&encrypted_bytes, &(output_filename.to_owned() + ".age"))
}

pub fn decrypt_file(key: age::x25519::Identity, target_file_name: &str) -> std::io::Result<()> {
    let target_file = fs::read(target_file_name)?;
    let decrypted = decrypt_bytes(key, target_file);
    let output_filename = Path::new(target_file_name)
        .file_stem() // strip the .age extenion
        .unwrap()
        .to_str()
        .unwrap();

    write_file_to_system(&decrypted, output_filename)
}

pub fn encrypt_dir(pubkey: age::x25519::Recipient, target_file_name: &str) -> std::io::Result<()> {
    // Writing a plaintext tar file to the file system is a potential security issue.
    // But at least this temporary tar file is created in the same
    // directory as the directory that we're bottling, NOT in the current
    // working directory, which user likely does NOT
    // want an unencrypted tar file to exists, even momentarily.
    let temp_tar_file_path = target_file_name.to_owned() + "_tarfile.tar";

    make_tar_from_dir(target_file_name, &temp_tar_file_path)
        .expect("Unable to make tar from given directory");
    // Now we compress the temp_tar_file_path with gzip
    let tar_file_as_bytes = fs::read(&temp_tar_file_path)?;
    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(&tar_file_as_bytes)?;
    let compressed_bytes = e.finish()?;

    let encrypted_bytes = encrypt_bytes(pubkey, &compressed_bytes);

    // Clean up
    fs::remove_file(&temp_tar_file_path)?;

    let output_name = parse_output_name(target_file_name);
    write_file_to_system(&encrypted_bytes, &(output_name + ".tar.gz.age"))
}

pub fn decrypt_dir(key: age::x25519::Identity, target_file_name: &str) -> std::io::Result<()> {
    let target_file = fs::read(target_file_name)?;
    let decrypted_bytes = decrypt_bytes(key, target_file);

    // At this point, decrypted_bytes needs to be decompressed.
    let mut d = GzDecoder::new(&*decrypted_bytes);
    let mut bytes = vec![];
    d.read_to_end(&mut bytes).expect("Error uncompressing file");
    write_file_to_system(&bytes, "_decrypted.tar")?;

    // Finally, we untar the file.
    let file = File::open("_decrypted.tar")?;
    let mut a = Archive::new(file);

    fs::remove_file("_decrypted.tar")?;

    let output_name = parse_output_name(target_file_name);
    // https://docs.rs/tar/latest/tar/struct.Archive.html#method.unpack
    a.unpack(output_name)
}

fn make_tar_from_dir(dir_name: &str, tar_name: &str) -> Result<(), std::io::Error> {
    let file = File::create(tar_name).unwrap();
    let mut a = Builder::new(file);

    // Use the directory at one location, but insert it into the archive
    // with a different name.
    // https://docs.rs/tar/latest/tar/struct.Builder.html#method.append_dir_all
    a.append_dir_all(".", dir_name).unwrap();

    a.finish()
}

fn write_file_to_system(data: &[u8], file_name: &str) -> std::io::Result<()> {
    let mut file = File::create(file_name)?;
    file.write_all(data)?;
    Ok(())
}

fn parse_output_name(target_file_name: &str) -> String {
    let file_name_without_extension = Path::new(target_file_name)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    let file_name_without_extension = split_and_vectorize(file_name_without_extension, ".")[0];
    file_name_without_extension.to_string()
}

// I found myself often wanting to split a string slice (`&str`)
// by another string slice and get a vector back.
pub fn split_and_vectorize<'a>(string_to_split: &'a str, splitter: &str) -> Vec<&'a str> {
    // let split = string_to_split.split(splitter);
    // split.collect::<Vec<&str>>()
    string_to_split.split(splitter).collect()
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn can_encrypt_and_decrypt_a_txt_file_harder() {
        let file_name_to_encrypt = "test-files/plain.txt";

        // Set up hard-coded key
        const KEY_FILE: &str = "test-files/key.txt";
        let key = read_key_from_file(KEY_FILE);

        // Create plain.txt.age
        encrypt_file(key.clone(), &file_name_to_encrypt).unwrap();
        // Decrypt plain.txt.age to plain.txt
        decrypt_file(key, "plain.txt.age").unwrap();

        // Now read the contents of the original clear text file...
        let original_contents = fs::read_to_string("test-files/plain.txt")
            .expect("Something went wrong reading the file");
        // ... and the content of the decrypted file
        let decrypted_contents =
            fs::read_to_string("plain.txt").expect("Something went wrong reading the file");
        // And compared them
        assert_eq!(original_contents, decrypted_contents);
        // Clean up working directory
        fs::remove_file("plain.txt").unwrap();
        fs::remove_file("plain.txt.age").unwrap();
    }

    #[test]
    fn can_encrypt_and_decrypt_the_bytes_of_a_txt_file() {
        let file_name_to_encrypt = "test-files/plain.txt";
        let file_to_encrypt =
            fs::read(file_name_to_encrypt).expect("Unable to read file to encrypt");

        // Set up hard-coded key
        const KEY_FILE: &str = "test-files/key.txt";
        let key = read_key_from_file(KEY_FILE);
        let pubkey = key.to_public();

        // Encrypt the plaintext to a ciphertext...
        let encrypted_bytes = encrypt_bytes(pubkey, &file_to_encrypt);

        // ... and decrypt the obtained ciphertext to the plaintext again.
        let decrypted = decrypt_bytes(key, encrypted_bytes);

        assert_eq!(decrypted, file_to_encrypt);
    }

    #[test]
    fn can_encrypt_and_decrypt_a_directory() {
        // Set up hard-coded key
        const KEY_FILE: &str = "test-files/key.txt";
        let key = read_key_from_file(KEY_FILE);
        let pubkey = key.to_public();

        // Declare our test dir
        let dir_name_to_encrypt = "test-files/test-dir";

        // this should create a `test-dir.tar.age` in WORKING directory
        encrypt_dir(pubkey, dir_name_to_encrypt).unwrap();

        // this should create a `test-dir` in WORKING directory
        // decrypt_dir(key, &(dir_name_to_encrypt.to_owned() + ".tar.age"));
        decrypt_dir(key, "test-dir.tar.gz.age").unwrap();

        // Finally, here's the test:
        // Read `./test-dir/file.txt` and make sure it includes the plaintext
        let contents =
            fs::read_to_string("test-dir/file.txt").expect("Something went wrong reading the file");
        assert_eq!(contents, "This is a file.\n");

        // Clean up working directory
        fs::remove_file("test-dir.tar.gz.age").unwrap();
        fs::remove_dir_all("test-dir").unwrap();
    }

    #[test]
    fn can_get_target_output_name() {
        // file
        assert_eq!(parse_output_name("test.txt"), "test");
        // directory
        assert_eq!(parse_output_name("foor/bar"), "bar");
        // longer rel path
        assert_eq!(parse_output_name("foo/bar/test.tar.gz.age"), "test");
        // Absolute path
        assert_eq!(
            parse_output_name("/home/user/foo/baz/test.tar.gz.age"),
            "test"
        );
    }
}
