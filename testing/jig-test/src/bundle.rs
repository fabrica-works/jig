//! Bounded bundle reads before the service loop (protocol/service.md, 7).
use jig_test_service::server::{self, Bundle, File};
use skein_lib::Map;
use std::{
    fs,
    os::unix::ffi::OsStrExt,
    path::{Component, Path},
};

const FILE_BYTES: u32 = 4 << 20;
const BUNDLE_BYTES: usize = 16 << 20;
const FILES: u32 = 128;
const ENTRIES: usize = 256;

fn refusal(path: &Path, why: impl std::fmt::Display) -> String {
    format!("bundle {}: {why}", path.display())
}
fn read_file(path: &Path, cap: u32) -> Result<Box<[u8]>, String> {
    skein_shell::read_file(path, cap).map_err(|why| refusal(path, format!("{why:?}")))
}
fn entries(directory: &Path, count: &mut usize) -> Result<Vec<fs::DirEntry>, String> {
    let mut found = Vec::new();
    for entry in fs::read_dir(directory).map_err(|why| refusal(directory, why))? {
        let entry = entry.map_err(|why| refusal(directory, why))?;
        *count += 1;
        if *count > ENTRIES {
            return Err(refusal(&entry.path(), "too many entries"));
        }
        found.push(entry);
    }
    Ok(found)
}

pub fn read(directory: &Path) -> Result<Bundle, String> {
    let page = read_file(&directory.join("index.html"), FILE_BYTES)?;
    let mut digest = None;
    for entry in fs::read_dir(directory).map_err(|why| refusal(directory, why))? {
        let entry = entry.map_err(|why| refusal(directory, why))?;
        let path = entry.path();
        let kind = entry.file_type().map_err(|why| refusal(&path, why))?;
        if kind.is_dir() {
            if digest.is_some() {
                return Err(refusal(&path, "more than one digest directory"));
            }
            let name = entry.file_name();
            let bytes = name.as_bytes();
            if bytes.len() != 16 || !bytes.iter().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) {
                return Err(refusal(&path, "digest must be 16 lowercase hex digits"));
            }
            digest = Some(Box::<[u8]>::from(bytes));
        } else if !kind.is_file() {
            return Err(refusal(&path, "entry must be a regular file or directory"));
        }
    }
    let digest = digest.ok_or_else(|| refusal(directory, "no digest directory"))?;
    let root = directory.join(std::ffi::OsStr::from_bytes(&digest));
    let mut pending = vec![root.clone()];
    let mut count = 0;
    let mut files = Map::with_capacity(FILES);
    let mut bytes = page.len();
    while let Some(directory) = pending.pop() {
        for entry in entries(&directory, &mut count)? {
            let path = entry.path();
            let name = path.strip_prefix(&root).map_err(|why| refusal(&path, why))?;
            if name.as_os_str().len() > 1024 || !name.components().all(|part| matches!(part, Component::Normal(_))) {
                return Err(refusal(&path, "path must be plain and at most 1024 bytes"));
            }
            let kind = entry.file_type().map_err(|why| refusal(&path, why))?;
            if kind.is_dir() {
                pending.push(path);
            } else if kind.is_file() {
                let name = name.to_str().ok_or_else(|| refusal(&path, "path is not UTF-8"))?.as_bytes();
                let kind = server::kind(name).ok_or_else(|| refusal(&path, "file type is not on the closed list"))?;
                let cap = if !name.contains(&b'/') && name.ends_with(b"_bg.wasm") {
                    u32::try_from(BUNDLE_BYTES).expect("bundle byte limit fits u32")
                } else {
                    FILE_BYTES
                };
                let content = read_file(&path, cap)?;
                bytes += content.len();
                if bytes > BUNDLE_BYTES || files.len() >= FILES {
                    return Err(refusal(&path, "bundle exceeds its limits"));
                }
                files.insert(Box::from(name), File { kind, bytes: content }).expect("file bound checked");
            } else {
                return Err(refusal(&path, "entry must be a regular file or directory"));
            }
        }
    }
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in &page {
        hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
    }
    Ok(Bundle { page, etag: format!("\"{hash:016x}\"").into_bytes().into_boxed_slice(), digest, files })
}
