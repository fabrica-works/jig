//! A frame patch reads only that frame, named by its places key and offset.
use alloc::boxed::Box;
#[derive(Default, Debug)]
pub struct Context {
    pub key: Box<[u8]>,
}
