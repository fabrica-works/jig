//! Startup file names and bounds; this crate performs no reads (configuration.md, 7).
use crate::{Engine, Path, Segment, Worker, llm::Transport};
use alloc::boxed::Box;
use skein_lib::{List, bytes};
#[derive(Debug)]
pub struct File {
    pub at: Path,
    pub path: Box<[u8]>,
    pub max: u32,
    pub private: bool,
}
#[must_use]
pub fn files(engine: &Engine) -> Box<[File]> {
    let max = engine
        .llm
        .providers
        .len()
        .checked_add(engine.llm.endpoints.len())
        .expect("bounded endpoints")
        .checked_add(engine.hosts.workers.len())
        .expect("bounded workers")
        .checked_add(engine.charters.len())
        .expect("bounded charters");
    let max = match &engine.clients.provider {
        Some(provider) => {
            max.checked_add(provider.clients.len()).expect("bounded clients").checked_add(1).expect("key file")
        }
        None => max,
    };
    let mut files = List::with_capacity(u32::try_from(max).expect("document count fits u32"));
    crate::charters::files(engine, &mut files);
    if let Some(provider) = &engine.clients.provider {
        let at = Path::root().member(b"clients").member(b"provider");
        push(&mut files, at.member(b"key"), &provider.key, 8192, true);
        for (index, client) in provider.clients.iter().enumerate() {
            let at = at
                .member(b"clients")
                .child(Segment::Index(u32::try_from(index).expect("bounded clients")))
                .member(b"secret");
            push(&mut files, at, &client.secret, 256, true);
        }
    }
    for (index, worker) in engine.hosts.workers.iter().enumerate() {
        let at = Path::root()
            .member(b"hosts")
            .member(b"workers")
            .child(Segment::Index(u32::try_from(index).expect("bounded workers")))
            .member(b"digest");
        push(&mut files, at, &worker.digest, 65, true);
    }
    for (index, endpoint) in engine.llm.endpoints.iter().enumerate() {
        let at = Path::root()
            .member(b"llm")
            .member(b"endpoints")
            .child(Segment::Index(u32::try_from(index).expect("bounded endpoints")));
        trust(&mut files, &endpoint.transport, &at);
    }
    for (index, provider) in engine.llm.providers.iter().enumerate() {
        if let Some(registration) = &provider.sign_in {
            let at = Path::root()
                .member(b"llm")
                .member(b"providers")
                .child(Segment::Index(u32::try_from(index).expect("bounded providers")))
                .member(b"sources")
                .member(b"sign_in");
            trust(&mut files, &registration.endpoint_transport, &at);
        }
    }
    files.into_boxed()
}
fn trust(files: &mut List<File>, transport: &Transport, at: &Path) {
    match transport {
        Transport::Plaintext => {}
        Transport::Tls { server_name: _, trust } => {
            if let Some(file) = trust {
                push(files, at.member(b"trust"), file, 1_048_576, false);
            }
        }
    }
}
/// The worker's startup reads its secret privately and leaves agent parsing to smith.
#[must_use]
pub fn worker_files(worker: &Worker) -> Box<[File]> {
    let mut files = List::with_capacity(2);
    push(&mut files, Path::root().member(b"secret"), &worker.secret, worker.limits.link.secret.saturating_add(1), true);
    push(
        &mut files,
        Path::root().member(b"agent").member(b"configuration"),
        &worker.agent.configuration,
        1_048_576,
        false,
    );
    files.into_boxed()
}
pub(crate) fn push(files: &mut List<File>, at: Path, path: &[u8], max: u32, private: bool) {
    let file = File { at, path: bytes::copy_of(path), max, private };
    files.push(file).expect("capacity includes every configured file");
}
