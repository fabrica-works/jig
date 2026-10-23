//! Client heartbeat and local sign-in link (configuration.md, 3).
use crate::{Directories, Path, Problems, check, read};
mod issuer;
mod provider;
use alloc::boxed::Box;
pub use issuer::{ProviderUrls, provider_issuer};
pub use provider::{Provider, ProviderClient, provider_key};
use skein_json::Document;
use skein_lib::{Duration, List};
#[derive(Debug)]
pub struct Clients {
    pub heartbeat: Duration,
    pub link: bool,
    pub provider: Option<Provider>,
}
pub(crate) fn parse(
    document: &Document,
    at: u32,
    path: &Path,
    directories: &Directories,
    problems: &mut Problems,
) -> Option<Clients> {
    let mut object = check::object(document, at, path, problems)?;
    let heartbeat = check::duration(&mut object, document, b"heartbeat_ms", path, problems);
    let mut link = None;
    if let Some(at) = object.required(document, b"link", path, problems) {
        match read::boolean(document, at) {
            Ok(value) => link = Some(value),
            Err(what) => problems.push(path.member(b"link"), what),
        }
    }
    let provider = provider::parse(&mut object, document, path, directories, problems);
    object.finish(document, path, problems);
    Some(Clients { heartbeat: heartbeat?, link: link?, provider })
}
/// The configured models as the client protocol names and shows them, in configuration's order,
/// each with the number of the LLM provider listing it (contract.md, 5.1).
#[must_use]
pub fn models(llm: &crate::Llm) -> Box<[jig_protocol_engine_clients::Model]> {
    let mut models = List::with_capacity(u32::try_from(llm.models.len()).expect("checked models"));
    for (at, model) in llm.models.iter().enumerate() {
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
                prices: jig_protocol_engine_clients::Prices {
                    input: model.input_price,
                    cached: model.cached_price,
                    cache_write: model.cache_write_price,
                    output: model.output_price,
                    unit: model.price_unit,
                },
                description: llm.descriptions.get(at).expect("one description per configured model").clone(),
            })
            .expect("one row per model");
    }
    models.into_boxed()
}

pub(crate) fn check_provider(engine: &crate::Engine, problems: &mut Problems) {
    let Some(provider) = &engine.clients.provider else { return };
    let path = Path::root().member(b"clients").member(b"provider").member(b"addresses");
    for (index, number) in provider.addresses.iter().enumerate() {
        let mut application = false;
        for configured in &engine.providers {
            application |= configured.number == *number && configured.kind == crate::ProviderKind::Application;
        }
        if !application {
            problems.push(check::index(&path, index), crate::What::Refers);
        }
    }
}
