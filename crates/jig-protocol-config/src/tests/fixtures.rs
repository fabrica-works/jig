use crate::{DocumentLimits, Path};
use skein_json::Document;
pub const LIMITS: DocumentLimits = DocumentLimits { bytes: 256, tokens: 128, depth: 8 };
pub const DIGEST_EMPTY: &[u8] = b"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n";
pub const SECRET_A: &[u8] = b"secret-a\n";
pub fn document(bytes: &[u8]) -> Document {
    crate::document(bytes, &LIMITS).expect("fixture JSON")
}
pub fn path(segments: &[&[u8]]) -> Path {
    let mut path = Path::root();
    for segment in segments {
        path = path.member(segment);
    }
    path
}
