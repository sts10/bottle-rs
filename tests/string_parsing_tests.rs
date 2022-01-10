mod string_parsing_tests {
    use bottle::*;
    #[test]
    fn can_get_target_output_name() {
        // file
        assert_eq!(parse_output_name("test.txt"), "test");
        // directory
        assert_eq!(parse_output_name("foo/bar"), "bar");
        // longer rel path
        assert_eq!(parse_output_name("foo/bar/test.tar.gz.age"), "test");
        // Absolute path
        assert_eq!(
            parse_output_name("/home/user/foo/baz/test.tar.gz.age"),
            "test"
        );
    }
}
