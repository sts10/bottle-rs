use bottle::*;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

/// bottle: Compress and encrypt (and decrypt and extract) files or directories using age, gzip, and tar.
#[derive(StructOpt, Debug)]
#[structopt(name = "bottle")]
struct Opt {
    /// Force overwrite when creating a file or directory
    #[structopt(short = "f", long = "force")]
    force_overwrite: bool,

    /// File or directory to either encrypt or decrypt.
    /// If given a directory, will tar, then gzip (compress), then encrypt, creating a file with
    /// the extension .tar.gz.age.
    /// If given a .tar.gz.age file, will decrypt and extract contents.
    /// All outputted files are placed in the current working directory.
    #[structopt(name = "TARGET", parse(from_os_str))]
    target_file: PathBuf,
}

enum Action {
    EncryptFile,
    DecryptFile,
    EncryptDir,
    DecryptDir,
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
    // make ~/.bottle directory if needed
    fs::create_dir_all(home_dir.to_str().unwrap().to_owned() + "/.bottle")?;
    // Create a key pair if needed
    generate_key_pair_if_none_exists(&key_file_location);

    let key = read_key_from_file(&key_file_location);
    let pubkey = key.to_public();

    let opt = Opt::from_args();
    // I'm sure we can do this better...
    let target_file_name = opt.target_file.to_str().unwrap();

    // Gather some data we'll need to determine what action
    // to take
    let metadata = fs::metadata(target_file_name)?;
    let is_dir = metadata.file_type().is_dir();
    let extension: Option<&str> = if is_dir {
        None
    } else {
        Path::new(target_file_name).extension().unwrap().to_str()
    };

    // Using the target_file_name, determine what action to take
    let action_to_take: Action = if is_dir {
        // Given a directory. We need to tar it, gzip it, then encrypt it
        Action::EncryptDir
    } else if target_file_name.ends_with(".tar.gz.age") {
        // If it's an encrypted, gzipped, and tar'd file...
        // assume it's a "bottle" directory we want to decrypt
        // and extract
        Action::DecryptDir
    } else if extension == Some("age") {
        // If extension is age, we assume it's an encrypted age file
        // that user wants to decrypt
        Action::DecryptFile
    } else {
        // Else, it's a regular, unencrypted file user
        // wants to encrypt with age key
        Action::EncryptFile
    };

    if !opt.force_overwrite && output_file_exists(target_file_name, &action_to_take) {
        if is_dir {
            // If given a directory, that means Bottle is being asked to make a file.
            // If we're here, that means that file already exists, and user didn't give the
            // --force flag.
            eprintln!("This command would overwrite an existing file. To do this, re-run with --force flag");
            // Err(ErrorKind::AlreadyExists)
            return Err(Error::new(ErrorKind::Other, "File exists"));
        } else {
            eprintln!("This command would overwrite an existing directory. To do this, re-run with --force flag");
            // Err(ErrorKind::AlreadyExists)
            return Err(Error::new(ErrorKind::Other, "Directory exists"));
        }
    } else {
        // If we're here, we know we don't need to worry about the output file
        // overwriting an existing file. Either there isn't a file at the path we're
        // going to use OR the user has used the --force flag and we don't care if
        // we overwrite it.
        match action_to_take {
            Action::EncryptDir => encrypt_dir(pubkey, target_file_name),
            Action::DecryptDir => decrypt_dir(key, target_file_name),
            Action::EncryptFile => encrypt_file(key, target_file_name),
            Action::DecryptFile => decrypt_file(key, target_file_name),
        }
    }
}

fn output_file_exists(target_file_name: &str, action_to_take: &Action) -> bool {
    let target_file_name_as_string = Path::new(target_file_name)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    // let file_name_without_extension =
    //     split_and_vectorize(&target_file_name_as_string, ".")[0].to_string();

    let target_file_name_minus_first_extension = Path::new(target_file_name)
        .file_stem() // strip the .age extenion
        .unwrap()
        .to_str()
        .unwrap();

    let file_name_without_extensions =
        split_and_vectorize(&target_file_name_as_string, ".")[0].to_string();

    match action_to_take {
        Action::EncryptFile => Path::new(&(target_file_name_as_string + ".age")).exists(),
        Action::EncryptDir => Path::new(&(target_file_name_as_string + ".tar.gz.age")).exists(),
        Action::DecryptFile => Path::new(&target_file_name_minus_first_extension).exists(),
        Action::DecryptDir => Path::new(&file_name_without_extensions).exists(),
    }
}
