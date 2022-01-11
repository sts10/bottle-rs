mod integration_tests {
    use bottle::*;
    use std::fs;

    #[test]
    fn can_encrypt_and_decrypt_the_bytes_of_a_txt_file() {
        let file_name_to_encrypt = "tests/test-files/plain.txt";
        let file_to_encrypt =
            fs::read(file_name_to_encrypt).expect("Unable to read file to encrypt");

        // Set up hard-coded key
        const KEY_FILE: &str = "tests/test-files/key.txt";
        let key = read_key_from_file(KEY_FILE);
        let pubkey = key.to_public();

        // Encrypt the plaintext to a ciphertext...
        let encrypted_bytes = encrypt_bytes(pubkey, &file_to_encrypt);

        // ... and decrypt the obtained ciphertext to the plaintext again.
        let decrypted = decrypt_bytes(key, encrypted_bytes);

        assert_eq!(decrypted, file_to_encrypt);
    }

    #[test]
    fn can_encrypt_and_decrypt_a_txt_file() {
        let file_name_to_encrypt = "tests/test-files/plain.txt";
        let file_name_for_encrypted_file = "tests/test-files/plain.txt.age";
        let file_name_for_decrypted_file = "tests/test-files/plain_decryped.txt";

        // Set up hard-coded key
        const KEY_FILE: &str = "tests/test-files/key.txt";
        let key = read_key_from_file(KEY_FILE);

        // Create plain.txt.age
        encrypt_file(
            key.to_public(),
            &file_name_to_encrypt,
            file_name_for_encrypted_file,
        )
        .unwrap();
        // Decrypt plain.txt.age to plain.txt
        decrypt_file(
            key,
            file_name_for_encrypted_file,
            file_name_for_decrypted_file,
        )
        .unwrap();

        // Now read the contents of the original clear text file...
        let original_contents = fs::read_to_string(file_name_to_encrypt)
            .expect("Something went wrong reading the file");
        // ... and the content of the decrypted file
        let decrypted_contents = fs::read_to_string(file_name_for_decrypted_file)
            .expect("Something went wrong reading the file");
        // And compared them
        assert_eq!(original_contents, decrypted_contents);
        // Clean up working directory
        fs::remove_file(file_name_for_encrypted_file).unwrap();
        fs::remove_file(file_name_for_decrypted_file).unwrap();
    }

    #[test]
    fn can_encrypt_and_decrypt_a_directory() {
        // Set up hard-coded key
        const KEY_FILE: &str = "tests/test-files/key.txt";
        let key = read_key_from_file(KEY_FILE);
        let pubkey = key.to_public();

        // Declare our test dir
        let dir_name_to_encrypt = "tests/test-files/test-dir";
        let dir_name_when_encrypted = "tests/test-files/test-dir.tar.gz.age";
        let dir_name_when_decrypted = "tests/test-files/test-dir_decrypted";

        // this should create a `test-dir.tar.age` in WORKING directory
        encrypt_dir(pubkey, dir_name_to_encrypt, dir_name_when_encrypted).unwrap();

        // this should create a `test-dir` in WORKING directory
        // decrypt_dir(key, &(dir_name_to_encrypt.to_owned() + ".tar.age"));
        decrypt_dir(key, dir_name_when_encrypted, dir_name_when_decrypted).unwrap();

        // Finally, here's the test:
        let original_contents = fs::read_to_string(dir_name_to_encrypt.to_owned() + "/file.txt")
            .expect("Something went wrong reading the file");
        let decrypted_contents =
            fs::read_to_string(dir_name_when_decrypted.to_owned() + "/file.txt")
                .expect("Something went wrong reading the file");
        assert_eq!(original_contents, decrypted_contents);
        assert_eq!(decrypted_contents, "This is a file.\n");

        // Clean up working directory
        fs::remove_file(dir_name_when_encrypted).unwrap();
        fs::remove_dir_all(dir_name_when_decrypted).unwrap();
    }
}
