//! Client heartbeat and local sign-in link (configuration.md, 3).
use crate::{Path, Problems, check, read};
use alloc::boxed::Box;
use skein_json::Document;
use skein_lib::{Duration, List};
#[derive(Debug)]
pub struct Clients {
    pub heartbeat: Duration,
    pub link: bool,
}
pub(crate) fn parse(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Clients> {
    let mut object = check::object(document, at, path, problems)?;
    let heartbeat = check::duration(&mut object, document, b"heartbeat_ms", path, problems);
    let mut link = None;
    if let Some(at) = object.required(document, b"link", path, problems) {
        match read::boolean(document, at) {
            Ok(value) => link = Some(value),
            Err(what) => problems.push(path.member(b"link"), what),
        }
    }
    object.finish(document, path, problems);
    Some(Clients { heartbeat: heartbeat?, link: link? })
}
/// The configured models as the client protocol names and shows them, in configuration's order,
/// each with the number of the LLM provider listing it (contract.md, 5.1).
#[must_use]
pub fn models(llm: &crate::Llm) -> Box<[jig_protocol_engine_clients::Model]> {
    let mut models = List::with_capacity(u32::try_from(llm.models.len()).expect("checked models"));
    for model in &llm.models {
        let mut provider = None;
        for item in &llm.providers {
            for name in &item.models {
                if *name == model.name {
                    provider = Some(item.number);
                }
            }
        }
        models
            .push(jig_protocol_engine_clients::Model {
                name: model.name.clone(),
                provider,
                window: model.window,
                output: model.output,
                efforts: model.efforts.clone(),
            })
            .expect("one row per model");
    }
    models.into_boxed()
}
