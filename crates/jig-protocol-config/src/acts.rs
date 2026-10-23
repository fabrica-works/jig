//! Optional connector grants for deployment acts (configuration.md, 3; people.md, 5.3).
use crate::{Path, Problems, What, check, read};
use alloc::boxed::Box;
use jig_domain_people::{ActGrant, ActKind};
use skein_json::Document;
use skein_lib::List;

pub(crate) fn parse(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    connectors: &[u16],
    problems: &mut Problems,
) -> Box<[ActGrant]> {
    let Some(at) = object.member(document, b"acts") else {
        return Box::new([]);
    };
    let path = path.member(b"acts");
    let items = match read::list(document, at, document.len()) {
        Ok(items) => items,
        Err(what) => {
            problems.push(path, what);
            return Box::new([]);
        }
    };
    let mut grants = List::<ActGrant>::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        let path = check::index(&path, index);
        if let Some(grant) = grant(document, *at, &path, problems) {
            if !connectors.contains(&grant.connector) {
                problems.push(path.member(b"connector"), What::Refers);
            }
            for previous in &grants {
                if previous.connector == grant.connector {
                    problems.push(path.member(b"connector"), What::Unique);
                }
            }
            grants.push(grant).expect("one per input object");
        }
    }
    grants.into_boxed()
}
fn grant(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<ActGrant> {
    let mut object = check::object(document, at, path, problems)?;
    let connector = check::short(&mut object, document, b"connector", path, problems);
    let items = check::list(&mut object, document, b"may", document.len(), path, problems);
    object.finish(document, path, problems);
    let items = items?;
    let mut may = List::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        let path = check::index(&path.member(b"may"), index);
        let name = match read::text(document, *at, 32) {
            Ok(name) => name,
            Err(what) => {
                problems.push(path, what);
                continue;
            }
        };
        let kind = match &*name {
            b"close" => ActKind::Close,
            b"add_key" => ActKind::AddKey,
            b"remove_account" => ActKind::RemoveAccount,
            b"adopt" => ActKind::Adopt,
            b"make_project" => ActKind::MakeProject,
            _ => {
                problems.push(path, What::Form);
                continue;
            }
        };
        if may.as_slice().contains(&kind) {
            problems.push(path, What::Unique);
        } else {
            may.push(kind).expect("one per input kind");
        }
    }
    Some(ActGrant { connector: connector?, may: may.into_boxed() })
}
