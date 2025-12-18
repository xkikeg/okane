use okane_golden::Golden;

use pretty_assertions::assert_str_eq;
use tempfile::{tempdir_in, NamedTempFile};

fn tempfile() -> std::io::Result<NamedTempFile> {
    NamedTempFile::new_in(env!("CARGO_TARGET_TMPDIR"))
}

mod update_golden_unset {
    use super::*;

    use regex::Regex;

    #[test]
    fn new_fails_on_non_existing_file() {
        temp_env::with_var_unset("UPDATE_GOLDEN", || {
            let dir = tempdir_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
            let got_err =
                Golden::new(dir.path().join("not_existing.txt")).expect_err("this must fail");

            assert!(Regex::new("Golden file .* not found")
                .unwrap()
                .is_match(&got_err.to_string()));

            dir.close().unwrap();
        });
    }

    #[test]
    fn assert_succeeds_on_correct_golden() {
        temp_env::with_var_unset("UPDATE_GOLDEN", || {
            let golden_file = tempfile().unwrap();
            let golden_path = golden_file.path();
            std::fs::write(golden_path, b"The quick fox").expect("golden file creation failed");

            let golden = Golden::new(golden_path.to_owned()).unwrap();
            golden.assert("The quick fox");
        });
    }

    #[test]
    fn assert_fails_on_different_golden() {
        temp_env::with_var_unset("UPDATE_GOLDEN", || {
            let golden_file = tempfile().unwrap();
            let golden_path = golden_file.path();
            std::fs::write(golden_path, b"The quick fox").expect("golden file creation failed");

            let golden = Golden::new(golden_path.to_owned()).unwrap();
            let got_err = std::panic::catch_unwind(|| golden.assert("いろはにほへと"))
                .expect_err("this assertion must fail");

            let payload = &*got_err;
            if payload.is::<String>() {
                assert!(payload
                    .downcast_ref::<String>()
                    .unwrap()
                    .contains("assertion failed"));
            } else {
                panic!("unexpected type of assertion failure");
            }
        });
    }
}

mod update_golden_set {
    use super::*;

    #[test]
    fn assert_creates_golden_when_file_not_exists() {
        temp_env::with_var("UPDATE_GOLDEN", Some("1"), || {
            let golden_file = tempfile().unwrap();
            let golden_path = golden_file.path();
            let golden =
                Golden::new(golden_path.to_owned()).expect("golden creation should succeed");

            golden.assert("Veni, vidi, vici\n");

            let updated = std::fs::read_to_string(golden_path).unwrap();
            assert_str_eq!("Veni, vidi, vici\n", updated);
        });
    }

    #[test]
    fn assert_updates_golden_when_file_content_different() {
        temp_env::with_var("UPDATE_GOLDEN", Some("1"), || {
            let golden_file = tempfile().unwrap();
            let golden_path = golden_file.path();
            std::fs::write(golden_path, b"numbergirl\n").expect("golden file creation failed");

            let golden =
                Golden::new(golden_path.to_owned()).expect("golden creation should succeed");

            golden.assert("Zazen\nBoys\n\n");

            let updated = std::fs::read_to_string(golden_path).unwrap();
            assert_str_eq!("Zazen\nBoys\n\n", updated);
        });
    }
}
