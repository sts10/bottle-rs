mod integration_tests {
    use bottle::*;
    use std::fs;

    #[test]
    fn can_encrypt_and_decrypt_a_txt_file() {
        let file_name_to_encrypt = "tests/test-files/plain.txt";

        // Set up hard-coded key
        const KEY_FILE: &str = "tests/test-files/key.txt";
        let key = read_key_from_file(KEY_FILE);

        // Create plain.txt.age
        encrypt_file(key.to_public(), &file_name_to_encrypt).unwrap();
        // Decrypt plain.txt.age to plain.txt
        decrypt_file(key, "plain.txt.age").unwrap();

        // Now read the contents of the original clear text file...
        let original_contents = fs::read_to_string("tests/test-files/plain.txt")
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
    fn can_encrypt_and_decrypt_a_directory() {
        // Set up hard-coded key
        const KEY_FILE: &str = "tests/test-files/key.txt";
        let key = read_key_from_file(KEY_FILE);
        let pubkey = key.to_public();

        // Declare our test dir
        let dir_name_to_encrypt = "tests/test-files/test-dir";

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
}
