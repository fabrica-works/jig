//! Startup file names and bounds; this crate performs no reads (configuration.md, 7).
use crate::{
    Engine, Path, Segment, Worker,
    llm::{Source, Transport},
};
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
        .accounts
        .len()
        .checked_mul(3)
        .expect("bounded accounts")
        .checked_add(engine.llm.endpoints.len())
        .expect("bounded endpoints")
        .checked_add(engine.hosts.workers.len())
        .expect("bounded workers")
        .checked_add(engine.charters.len())
        .expect("bounded charters");
    let mut files = List::with_capacity(u32::try_from(max).expect("document count fits u32"));
    crate::charters::files(engine, &mut files);
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
    for (index, account) in engine.llm.accounts.iter().enumerate() {
        let at = Path::root()
            .member(b"llm")
            .member(b"accounts")
            .child(Segment::Index(u32::try_from(index).expect("bounded accounts")))
            .member(b"source");
        match &account.source {
            Source::HandedIn { file } => push(&mut files, at.member(b"handed_in").member(b"file"), file, 65_536, true),
            Source::SignIn(sign_in) => {
                let at = at.member(b"sign_in");
                if let Some(secret) = &sign_in.secret {
                    push(&mut files, at.member(b"secret"), secret, engine.limits.token, true);
                }
                trust(&mut files, &sign_in.transport, &at);
            }
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
