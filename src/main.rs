use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::iter;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

/// bottle: Archive tool
#[derive(StructOpt, Debug)]
#[structopt(name = "bottle")]
struct Opt {
    /// CSV of family names
    #[structopt(name = "TARGET", parse(from_os_str))]
    target_file: PathBuf,
}

fn main() {
    // Set up hard-coded key
    const KEY_FILE: &str = "key.txt";
    let key = read_key_from_file(KEY_FILE);
    let pubkey = key.to_public();

    let opt = Opt::from_args();
    // I'm sure we can do this better...
    let target_file_name = opt.target_file.to_str().unwrap();

    let is_dir = fs::metadata(target_file_name).unwrap().file_type().is_dir();

    if is_dir {
        println!("Making a tar!");
        make_tar_from_dir(target_file_name, "mytar.tar");
        println!("Done with make_tar_from_dir call");
    } else {
        let target_file = fs::read(target_file_name).expect("Unable to read file to encrypt");
        let extension = Path::new(target_file_name).extension().unwrap().to_str();

        if extension == Some("age") {
            // let encrypted_file_to_decrypt =
            let decrypted = decrypt_file(target_file, key);
            write_file_to_system(&decrypted, "decrypted.txt")
                .expect("Unable to write encrypted data to a file");
        } else {
            // Encrypt the plaintext to a ciphertext...
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

use tar::Builder;
fn make_tar_from_dir(dir_name: &str, tar_name: &str) -> Result<(), std::io::Error> {
    // https://docs.rs/tar/latest/tar/struct.Builder.html#method.append_dir_all
    // let mut ar = Builder::new(Vec::new());

    let file = File::create(tar_name).unwrap();
    let mut a = Builder::new(file);

    println!("Created Builder");
    // Use the directory at one location, but insert it into the archive
    // with a different name.
    a.append_dir_all(dir_name, ".").unwrap();

    println!("Done appending dir");
    a.finish();
    Ok(())
}

fn write_file_to_system(data: &Vec<u8>, file_name: &str) -> std::io::Result<()> {
    let mut file = File::create(file_name)?;
    file.write_all(data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn it_works() {
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
}
