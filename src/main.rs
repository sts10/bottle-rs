use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::iter;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;
use tar::Archive;

/// bottle: Archive tool
#[derive(StructOpt, Debug)]
#[structopt(name = "bottle")]
struct Opt {
    /// CSV of family names
    #[structopt(name = "TARGET", parse(from_os_str))]
    target_file: PathBuf,
}

// fn main() -> std::io::Result<()> {
fn main() {
    // Set up hard-coded key
    const KEY_FILE: &str = "key.txt";
    let key = read_key_from_file(KEY_FILE);
    let pubkey = key.to_public();

    let opt = Opt::from_args();
    // I'm sure we can do this better...
    let target_file_name = opt.target_file.to_str().unwrap();
    println!("target_file_name is {}", target_file_name);

    let metadata = fs::metadata(target_file_name).unwrap();
    let is_dir = metadata.file_type().is_dir();

    if is_dir {
        // Given a directory. We need to tar it, then encrypt it
        println!("Making a tar!");
        encrypt_dir(pubkey, target_file_name);
    } else if target_file_name.ends_with(".tar.age") {
        // If it's an encrypted and tar'd file...
        decrypt_dir(key, target_file_name);
    } else {
        // If we're here that means we were given a file.
        let target_file = fs::read(target_file_name).expect("Unable to read file to encrypt");
        let extension = Path::new(target_file_name).extension().unwrap().to_str();

        if extension == Some("age") {
            // If extension is age, we assume it's an encrypted age file
            // that user wants to decrypt
            let decrypted = decrypt_file(target_file, key);
            write_file_to_system(&decrypted, "decrypted.txt")
                .expect("Unable to write encrypted data to a file");
        } else {
            // Else, it's a regular, unencrypted file user
            // wants to encrypt with age key
            let encrypted = encrypt_file(pubkey, &target_file);

            write_file_to_system(&encrypted, "output.txt.age")
                .expect("Unable to write encrypted data to a file");
        }
    }

    println!("Done!");
}

fn read_key_from_file(file_name: &str) -> age::x25519::Identity {
    let identify_file_entry = age::IdentityFile::from_file(file_name.to_string())
        .expect("Error reading key from file")
        .into_identities();
    // Bummed about this clone(), but can't figure out another way
    // right now
    let key = match identify_file_entry[0].clone() {
        age::IdentityFileEntry::Native(i) => i,
        _ => unreachable!(),
    };
    key
}

fn encrypt_file(pubkey: age::x25519::Recipient, file_to_encrypt: &Vec<u8>) -> Vec<u8> {
    let encryptor = age::Encryptor::with_recipients(vec![Box::new(pubkey)]);

    let mut encrypted = vec![];
    let mut writer = encryptor.wrap_output(&mut encrypted).unwrap();
    // writer.write_all(plaintext).unwrap();
    writer.write_all(&file_to_encrypt).unwrap();
    writer.finish().unwrap();

    encrypted
}

fn decrypt_file(encrypted: Vec<u8>, key: age::x25519::Identity) -> Vec<u8> {
    let decryptor = match age::Decryptor::new(&encrypted[..]).unwrap() {
        age::Decryptor::Recipients(d) => d,
        _ => unreachable!(),
    };

    let mut decrypted = vec![];
    let mut reader = decryptor
        .decrypt(iter::once(&key as &dyn age::Identity))
        .unwrap();
    reader.read_to_end(&mut decrypted);

    decrypted
}

fn encrypt_dir(pubkey: age::x25519::Recipient, target_file_name: &str) {
    let output_name = parse_out_put_name(target_file_name);

    let tar_file_name = "_tarfile.tar";
    make_tar_from_dir(target_file_name, tar_file_name)
        .expect("Unable to make tar from given directory");
    println!("Done with make_tar_from_dir call");
    let encrypted = encrypt_file(pubkey, &fs::read(tar_file_name).unwrap());

    write_file_to_system(&encrypted, &(output_name.to_owned() + ".tar.age"))
        .expect("Unable to write encrypted data to a file");

    fs::remove_file("_tarfile.tar").unwrap();
}

fn decrypt_dir(key: age::x25519::Identity, target_file_name: &str) {
    let target_file = fs::read(target_file_name).expect("Unable to read file to encrypt");
    let decrypted = decrypt_file(target_file, key);
    write_file_to_system(&decrypted, "_decrypted.tar")
        .expect("Unable to write encrypted data to a file");

    let file = File::open("_decrypted.tar").unwrap();
    let mut a = Archive::new(file);

    let output_name = parse_out_put_name(target_file_name);
    // https://docs.rs/tar/latest/tar/struct.Archive.html#method.unpack
    a.unpack(output_name).unwrap();
    fs::remove_file("_decrypted.tar").unwrap();
}

use tar::Builder;
fn make_tar_from_dir(dir_name: &str, tar_name: &str) -> Result<(), std::io::Error> {
    // https://docs.rs/tar/latest/tar/struct.Builder.html#method.append_dir_all
    // let mut ar = Builder::new(Vec::new());

    let file = File::create(tar_name).unwrap();
    let mut a = Builder::new(file);

    println!("Created Builder");
    // Use the directory at one location, but insert it into the archive
    // with a different name.
    a.append_dir_all(".", dir_name).unwrap();

    println!("Done appending dir");
    a.finish();
    Ok(())
}

fn write_file_to_system(data: &Vec<u8>, file_name: &str) -> std::io::Result<()> {
    let mut file = File::create(file_name)?;
    file.write_all(data)?;
    Ok(())
}

fn parse_out_put_name(target_file_name: &str) -> String {
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
    fn can_encrypt_and_decrypt_a_txt_file() {
        let file_name_to_encrypt = "plain.txt";
        let file_to_encrypt =
            fs::read(file_name_to_encrypt).expect("Unable to read file to encrypt");

        // Set up hard-coded key
        const KEY_FILE: &str = "key.txt";
        let key = read_key_from_file(KEY_FILE);
        let pubkey = key.to_public();

        // Encrypt the plaintext to a ciphertext...
        let encrypted = encrypt_file(pubkey, &file_to_encrypt);

        // ... and decrypt the obtained ciphertext to the plaintext again.
        let decrypted = decrypt_file(encrypted, key);

        assert_eq!(decrypted, file_to_encrypt);
    }
    #[test]
    fn can_encrypt_and_decrypt_a_directory() {
        let dir_name_to_encrypt = "test-files/test-dir";

        // Set up hard-coded key
        const KEY_FILE: &str = "key.txt";
        let key = read_key_from_file(KEY_FILE);
        let pubkey = key.to_public();

        encrypt_dir(pubkey, dir_name_to_encrypt);
        fs::remove_dir_all(dir_name_to_encrypt).unwrap();
        decrypt_dir(key, encrypted);

        // assert_eq!(decrypted, file_to_encrypt);
    }
}
