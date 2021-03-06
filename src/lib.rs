use age::secrecy::ExposeSecret;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs;
use std::fs::File;
// use std::io::BufReader;
// use std::io::{Error, ErrorKind};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::iter;
use std::path::Path;
use tar::Archive;
use tar::Builder;

pub fn generate_key_pair_if_none_exists(key_file_path: &str) {
    if Path::new(key_file_path).exists() {
        // eprintln!(
        //     "Age Identity (key-pair) already exists at {}\n Won't create a new one.",
        //     key_file_path
        // );
    } else {
        eprintln!("No existing Age Identity (key-pair) found. Creating a new Age Identity for Bottle to use at {}", key_file_path);
        generate_key_pair_to_file(key_file_path);
    }
}

fn generate_key_pair_to_file(key_file_path: &str) {
    // Adapted from rage-keygen tool:
    // https://github.com/str4d/rage/blob/main/rage/src/bin/rage-keygen/main.rs#L87-L94
    let sk = age::x25519::Identity::generate();
    let pk = sk.to_public();
    eprintln!("Public key: {}", pk);

    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .open(key_file_path)
        .unwrap();
    writeln!(
        f,
        "# created: {}",
        chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    )
    .unwrap();
    writeln!(f, "# public key: {}", pk).unwrap();
    writeln!(f, "{}", sk.to_string().expose_secret()).unwrap();
}

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

pub fn encrypt_bytes(pubkey: age::x25519::Recipient, bytes_to_encrypt: &[u8]) -> Vec<u8> {
    let encryptor = age::Encryptor::with_recipients(vec![Box::new(pubkey)]);

    let mut encrypted_bytes = vec![];
    let mut writer = encryptor.wrap_output(&mut encrypted_bytes).unwrap();
    writer.write_all(bytes_to_encrypt).unwrap();
    writer.finish().unwrap();

    encrypted_bytes
}

pub fn decrypt_bytes(key: age::x25519::Identity, encrypted_bytes: Vec<u8>) -> Vec<u8> {
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

pub fn encrypt_file(
    pubkey: age::x25519::Recipient,
    target_file_name: &str,
    output_filename: &str,
) -> std::io::Result<()> {
    let target_file = fs::read(target_file_name)?;

    let encrypted_bytes = encrypt_bytes(pubkey, &target_file);

    write_file_to_system(&encrypted_bytes, output_filename)
}

pub fn decrypt_file(
    key: age::x25519::Identity,
    target_file_name: &str,
    output_filename: &str,
) -> std::io::Result<()> {
    let target_file = fs::read(target_file_name)?;
    let decrypted = decrypt_bytes(key, target_file);

    write_file_to_system(&decrypted, output_filename)
}

pub fn encrypt_dir(
    pubkey: age::x25519::Recipient,
    target_file_name: &str,
    output_filename: &str,
) -> std::io::Result<()> {
    // First, we convert the directory the user gave us into tarred bytes
    let tarred_bytes = make_tarred_bytes_from_dir(target_file_name)
        .expect("Unable to make tarred bytes from given directory");

    // Now we compress the tarred bytes with gzip
    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(&tarred_bytes)?;
    let compressed_bytes = e.finish()?;

    // Then we encrypt these compressed bytes with the age public key
    // we received.
    let encrypted_bytes = encrypt_bytes(pubkey, &compressed_bytes);

    // And finally write it to the file system as a .tar.gz.age file!
    write_file_to_system(&encrypted_bytes, output_filename)
}

fn _compose_path_for_temp_tar_file(target_file_name: &str, output_filename: &str) -> String {
    // For security reasons (see comment above) , we want to put our temp
    // tarfile in the parent directory of the directory we're bottling.
    // A possible alternative approach would be to put it in "/tmp/"?
    Path::new(target_file_name)
        .parent()
        .expect("Must have access to parent directory")
        .to_str()
        .unwrap()
        .to_owned()
        + "/_tarfile_created_by_bottle_for_"
        + &output_filename.replace(".", "-").replace("/", "_")
        + ".tar"
}

pub fn decrypt_dir(
    key: age::x25519::Identity,
    target_file_name: &str,
    output_dir_name: &str,
) -> std::io::Result<()> {
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

    // https://docs.rs/tar/latest/tar/struct.Archive.html#method.unpack
    a.unpack(output_dir_name)
}

fn make_tarred_bytes_from_dir(dir_name: &str) -> Result<Vec<u8>, std::io::Error> {
    // Create a buffer for us to Write into
    let b: Vec<u8> = Vec::new();
    // Create a tar::Builder thing
    let mut a = Builder::new(b);

    // Use the directory at one location, but insert it into the archive
    // with a different name.
    // https://docs.rs/tar/latest/tar/struct.Builder.html#method.append_dir_all
    a.append_dir_all(".", dir_name).unwrap();

    let tarred_bytes = a.into_inner();
    tarred_bytes
}

fn write_file_to_system(data: &[u8], file_name: &str) -> std::io::Result<()> {
    let mut file = File::create(file_name)?;
    file.write_all(data)?;
    Ok(())
}
