use std::io::{Read, Write};
use std::iter;
fn main() {
    const KEY_FILE: &str = "test.txt";
    let identify_file_entry = &age::IdentityFile::from_file(KEY_FILE.to_string())
        .expect("Error reading key from file")
        .into_identities()[0];
    let key = match &identify_file_entry {
        age::IdentityFileEntry::Native(i) => i,
        _ => unreachable!(),
    };
    let pubkey = key.to_public();

    let plaintext = b"Hello world!";

    // Encrypt the plaintext to a ciphertext...
    let encrypted = {
        let encryptor = age::Encryptor::with_recipients(vec![Box::new(pubkey)]);

        let mut encrypted = vec![];
        let mut writer = encryptor.wrap_output(&mut encrypted).unwrap();
        writer.write_all(plaintext).unwrap();
        writer.finish().unwrap();

        encrypted
    };

    // ... and decrypt the obtained ciphertext to the plaintext again.
    let decrypted = {
        let decryptor = match age::Decryptor::new(&encrypted[..]).unwrap() {
            age::Decryptor::Recipients(d) => d,
            _ => unreachable!(),
        };

        let mut decrypted = vec![];
        let mut reader = decryptor
            .decrypt(iter::once(key as &dyn age::Identity))
            .unwrap();
        reader.read_to_end(&mut decrypted);

        decrypted
    };

    assert_eq!(decrypted, plaintext);
    println!("Hello, world!");
}
