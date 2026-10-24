//! The shell sets the record key before each walk (codec.md 11.5).
use alloc::boxed::Box;

#[derive(Default, Debug)]
pub struct Context {
    pub key: Box<[u8]>,
}
