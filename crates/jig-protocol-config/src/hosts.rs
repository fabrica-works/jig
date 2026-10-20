//! Workers and their plaintext listeners (configuration.md, 3).
use crate::{Address, Directories, Directory, Limits, Path, Problems, Segment, What, address, check};
use alloc::boxed::Box;
use skein_json::Document;
use skein_lib::List;
#[derive(Debug)]
pub struct Hosts {
    pub workers: Box<[WorkerName]>,
    pub listener: Box<[Address]>,
}
#[derive(Debug)]
pub struct WorkerName {
    pub name: Box<[u8]>,
    pub digest: Box<[u8]>,
}
pub(crate) fn parse(
    document: &Document,
    at: u32,
    path: &Path,
    limits: &Limits,
    directories: &Directories,
    problems: &mut Problems,
) -> Option<Hosts> {
    let mut object = check::object(document, at, path, problems)?;
    let mut workers = List::<WorkerName>::with_capacity(limits.link.names);
    if let Some(items) = check::list(
        &mut object,
        document,
        b"workers",
        limits.link.names.min(limits.core.fleet.configured_workers),
        path,
        problems,
    ) {
        for (index, at) in items.iter().enumerate() {
            let path =
                path.member(b"workers").child(Segment::Index(u32::try_from(index).expect("list bounded by u32")));
            if let Some(worker) = worker(
                document,
                *at,
                &path,
                limits.link.name.min(limits.core.fleet.worker_name_bytes),
                directories,
                problems,
            ) {
                for previous in &workers {
                    if previous.name == worker.name {
                        problems.push(path.member(b"name"), What::Unique);
                        break;
                    }
                }
                workers.push(worker).expect("input count bounded by workers");
            }
        }
    }
    let mut listener = List::with_capacity(document.len());
    if let Some(items) = check::list(&mut object, document, b"listener", document.len(), path, problems) {
        for (index, at) in items.iter().enumerate() {
            let path =
                path.member(b"listener").child(Segment::Index(u32::try_from(index).expect("list bounded by u32")));
            if let Some(address) = address::read(document, *at, &path, problems) {
                check::local(&address, &path, problems);
                listener.push(address).expect("input count bounded by openings");
            }
        }
    }
    object.finish(document, path, problems);
    Some(Hosts { workers: workers.into_boxed(), listener: listener.into_boxed() })
}
fn worker(
    document: &Document,
    at: u32,
    path: &Path,
    max_name: u32,
    directories: &Directories,
    problems: &mut Problems,
) -> Option<WorkerName> {
    let mut object = check::object(document, at, path, problems)?;
    let name = check::text(&mut object, document, b"name", path, problems);
    if let Some(name) = &name
        && name.len() > usize::try_from(max_name).expect("byte bound fits usize")
    {
        problems.push(path.member(b"name"), What::Bound);
    }
    let digest = check::path(&mut object, document, b"digest", path, Directory::Data, directories, problems);
    object.finish(document, path, problems);
    Some(WorkerName { name: name?, digest: digest? })
}
