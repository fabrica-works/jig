use crate::{DocumentLimits, Path};
use skein_json::Document;
pub const LIMITS: DocumentLimits = DocumentLimits { bytes: 256, tokens: 128, depth: 8 };
pub const DIGEST_EMPTY: &[u8] = b"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n";
pub const SECRET_A: &[u8] = b"secret-a\n";
pub fn document(bytes: &[u8]) -> Document {
    crate::document(bytes, &LIMITS).expect("fixture JSON")
}
pub fn path(segments: &[&[u8]]) -> Path {
    let mut path = Path::root();
    for segment in segments {
        path = path.member(segment);
    }
    path
}

pub const ENGINE_SMALL: &[u8] = br#"{
  "limits": {"profile": "small"},
  "llm": {
    "endpoints": [{
      "name": "fake",
      "number": 1,
      "dialect": 0,
      "account": 1,
      "provider": "fake-provider",
      "address": {"socket": "127.0.0.1:7101"},
      "transport": "plaintext",
      "headers": [],
      "connect_ms": 10000,
      "handshake_ms": 10000
    }],
    "models": [{
      "name": "small",
      "endpoint": "fake",
      "window": 1024,
      "output": 128,
      "prices": {"input": 1, "cached": 1, "output": 2, "unit": 1000}
    }],
    "accounts": [{
      "number": 1,
      "source": {"handed_in": {"file": "accounts/chat.json"}}
    }],
    "records": "accounts/records"
  },
  "hosts": {
    "workers": [
      {"name": "worker-a", "digest": "secrets/worker-a.digest"},
      {"name": "worker-b", "digest": "secrets/worker-b.digest"}
    ],
    "listener": [{"in_process": "workers"}]
  },
  "clients": {
    "listener": {"socket": "127.0.0.1:7100"},
    "heartbeat_ms": 10000,
    "link": true
  }
}"#;
pub fn engine(bytes: &[u8]) -> Result<crate::Engine, crate::Problems> {
    let limits = DocumentLimits { bytes: 8192, tokens: 1024, depth: 16 };
    let document = match crate::document(bytes, &limits) {
        Ok(document) => document,
        Err(problem) => {
            let mut problems = crate::Problems::empty();
            problems.push(problem.path, problem.what);
            return Err(problems);
        }
    };
    crate::engine(&document, 0)
}
pub fn with(document: &[u8], from: &[u8], to: &[u8]) -> Box<[u8]> {
    let at = skein_lib::bytes::find(document, from).expect("fixture replacement exists");
    let end = at.checked_add(from.len()).expect("fixture fits usize");
    let len = document.len().checked_sub(from.len()).unwrap().checked_add(to.len()).unwrap();
    let mut writer = skein_lib::Writer::new(len);
    writer.put(document.get(..at).unwrap()).unwrap();
    writer.put(to).unwrap();
    writer.put(document.get(end..).unwrap()).unwrap();
    writer.finish()
}
pub fn refuses(from: &[u8], to: &[u8], path: Path, what: crate::What) {
    let bytes = with(ENGINE_SMALL, from, to);
    let problems = engine(&bytes).unwrap_err();
    assert_eq!(problems.problems.get(0), Some(&crate::Problem { path, what }));
}
