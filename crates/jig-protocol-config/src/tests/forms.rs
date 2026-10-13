use super::fixtures::{DIGEST_EMPTY, SECRET_A};
use crate::{What, forms};
use skein_io::digest::Digest;
use skein_lib::bytes;
const EMPTY: Digest =
    Digest([0xe3b0_c442_98fc_1c14, 0x9afb_f4c8_996f_b924, 0x27ae_41e4_649b_934c, 0xa495_991b_7852_b855]);
fn a_digest_file_with_uppercase_digits_is_refused() {
    let bytes = b"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855\n";
    assert_eq!(forms::digest(bytes), Err(What::Digest));
}
#[test]
fn a_digest_file_without_its_final_newline_is_read() {
    assert_eq!(forms::digest(&DIGEST_EMPTY[..64]), Ok(EMPTY));
}
fn a_digest_of_63_digits_is_refused() {
    assert_eq!(forms::digest(&DIGEST_EMPTY[..63]), Err(What::Digest));
}
fn a_digest_followed_by_more_than_a_newline_is_refused() {
    let bytes = b"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n\n";
    assert_eq!(forms::digest(bytes), Err(What::Digest));
}
fn a_secret_files_final_newline_is_dropped() {
    assert_eq!(forms::secret(bytes::copy_of(SECRET_A)).expose(), b"secret-a");
}
fn a_secret_keeps_an_inner_newline() {
    assert_eq!(forms::secret(bytes::copy_of(b"secret\na\n")).expose(), b"secret\na");
}

#[test]
fn malformed_digest_files_are_refused() {
    table([
        ("a_digest_file_with_uppercase_digits_is_refused", a_digest_file_with_uppercase_digits_is_refused),
        ("a_digest_of_63_digits_is_refused", a_digest_of_63_digits_is_refused),
        ("a_digest_followed_by_more_than_a_newline_is_refused", a_digest_followed_by_more_than_a_newline_is_refused),
    ]);
}

#[test]
fn secret_files_drop_only_the_final_newline() {
    table([
        ("a_secret_files_final_newline_is_dropped", a_secret_files_final_newline_is_dropped),
        ("a_secret_keeps_an_inner_newline", a_secret_keeps_an_inner_newline),
    ]);
}

type Case = (&'static str, fn());

fn table<const N: usize>(cases: [Case; N]) {
    let mut failures = [None; N];
    let mut passed = true;
    for ((name, case), failure) in cases.into_iter().zip(&mut failures) {
        if std::panic::catch_unwind(case).is_err() {
            *failure = Some(name);
            passed = false;
        }
    }
    assert!(passed, "{failures:?}");
}
