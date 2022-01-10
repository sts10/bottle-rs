use bottle::*;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

/// bottle: Compress and encrypt (and decrypt and extract files or directories using age, gzip, and tar.
#[derive(StructOpt, Debug)]
#[structopt(name = "bottle")]
struct Opt {
    /// File or directory to either encrypt or decrypt.
    /// If given a directory, will tar, then gzip (compress), then encrypt, creating a file with
    /// the extension .tar.gz.age.
    /// If given a .tar.gz.age file, will decrypt and extract contents.
    /// All outputted files are placed in the current working directory.
    #[structopt(name = "TARGET", parse(from_os_str))]
    target_file: PathBuf,
}

fn main() -> std::io::Result<()> {
    // Set up hard-coded key
    // Use the home crate to get user's $HOME directory
    let home_dir = match home::home_dir() {
        Some(path) => path,
        None => {
            panic!("Unable to find your HOME directory, and thus can not locate age key-pair file. Exiting.")
        }
    };
    let key_file_location = home_dir.to_str().unwrap().to_owned() + "/.bottle/bottle_key.txt";
    generate_key_pair_if_none_exists(&key_file_location);

    let key = read_key_from_file(&key_file_location);
    let pubkey = key.to_public();

    let opt = Opt::from_args();
    // I'm sure we can do this better...
    let target_file_name = opt.target_file.to_str().unwrap();

    let metadata = fs::metadata(target_file_name)?;
    let is_dir = metadata.file_type().is_dir();

    if is_dir {
        // Given a directory. We need to tar it, gzip it, then encrypt it
        encrypt_dir(pubkey, target_file_name)
    } else if target_file_name.ends_with(".tar.gz.age") {
        // If it's an encrypted and tar'd file...
        decrypt_dir(key, target_file_name)
    } else {
        // If we're here that means we were given a file.
        // Let's find the extension of the file so we know what
        // to do with it.
        let extension = Path::new(target_file_name).extension().unwrap().to_str();

        if extension == Some("age") {
            // If extension is age, we assume it's an encrypted age file
            // that user wants to decrypt
            decrypt_file(key, target_file_name)
        } else {
            // Else, it's a regular, unencrypted file user
            // wants to encrypt with age key
            encrypt_file(key, target_file_name)
        }
    }
}
