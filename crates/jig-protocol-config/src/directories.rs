//! The directories a document's relative paths are joined to (configuration.md, 10).
use crate::What;
use alloc::boxed::Box;
use skein_lib::{Writer, bytes};

/// A path member's kind, after the application's three directories.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Directory {
    /// What the operator writes and may copy or publish.
    Config,
    /// What the deployment keeps, secrets among it.
    Data,
    /// What can be fetched again.
    Cache,
}

/// The caller's directories, or none: then each relative path is kept as
/// written, for the kernel to resolve against the working directory.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Directories {
    given: Option<Given>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
struct Given {
    config: Box<[u8]>,
    data: Box<[u8]>,
    cache: Box<[u8]>,
}
impl Directories {
    /// The application's three directories. `None` unless each begins
    /// with `/` and is at most 4096 bytes.
    #[must_use]
    pub fn given(config: &[u8], data: &[u8], cache: &[u8]) -> Option<Self> {
        for directory in [config, data, cache] {
            if !directory.starts_with(b"/") || directory.len() > 4096 {
                return None;
            }
        }
        Some(Self {
            given: Some(Given {
                config: bytes::copy_of(config),
                data: bytes::copy_of(data),
                cache: bytes::copy_of(cache),
            }),
        })
    }

    /// No directories given, as `jig-test` and tests run.
    #[must_use]
    pub fn working() -> Self {
        Self { given: None }
    }

    /// A path member's value, resolved. An absolute path (beginning with
    /// `/`) is returned as written. A relative one must be non-empty with
    /// no component `..` (components split on `/`), else `What::Relative`;
    /// it is joined to its kind's directory with one `/` between them (no
    /// second `/` when the directory ends in one), or kept as written
    /// under `working()`. A result over 4096 bytes is `What::Bound`.
    pub fn join(&self, directory: Directory, path: &[u8]) -> Result<Box<[u8]>, What> {
        if path.starts_with(b"/") {
            return bounded(path);
        }
        if path.is_empty()
            || path == b".."
            || path.starts_with(b"../")
            || path.ends_with(b"/..")
            || bytes::find(path, b"/../").is_some()
        {
            return Err(What::Relative);
        }
        let Some(given) = &self.given else {
            return bounded(path);
        };
        let base = match directory {
            Directory::Config => &given.config,
            Directory::Data => &given.data,
            Directory::Cache => &given.cache,
        };
        let separator: &[u8] = if base.ends_with(b"/") { b"" } else { b"/" };
        let len = base.len().saturating_add(separator.len()).saturating_add(path.len());
        if len > 4096 {
            return Err(What::Bound);
        }
        let mut writer = Writer::new(len);
        writer.put(base).expect("joined length checked");
        writer.put(separator).expect("joined length checked");
        writer.put(path).expect("joined length checked");
        Ok(writer.finish())
    }
}
fn bounded(path: &[u8]) -> Result<Box<[u8]>, What> {
    if path.len() > 4096 {
        return Err(What::Bound);
    }
    Ok(bytes::copy_of(path))
}
