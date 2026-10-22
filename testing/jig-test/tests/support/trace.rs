//! Capture the provider's content without the credential or HTTP headers.
use std::{fs, path::Path};

pub fn settings(directory: &Path) -> serde_json::Value {
    serde_json::json!({
        "path":directory.join("agent-trace.jsonl"), "capture":"everything", "delivery":"complete"
    })
}

pub fn configure(directory: &Path) {
    let path = directory.join("agent.json");
    let mut agent: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("agent fixture")).expect("agent JSON");
    agent["trace"] = settings(directory);
    super::private(&path, &serde_json::to_vec(&agent).expect("traced agent JSON"));
}

pub fn check(directory: &Path, credential: Option<&[u8]>) {
    let bytes = fs::read(directory.join("agent-trace.jsonl")).expect("Smith provider trace");
    let mut prompts = 0;
    for line in bytes.split(|byte| *byte == b'\n').filter(|line| !line.is_empty()) {
        let record: serde_json::Value = serde_json::from_slice(line).expect("Smith event JSON");
        if record.get("prompt").is_some_and(serde_json::Value::is_object) {
            prompts += 1;
        }
        fields(&record, credential);
    }
    assert!(prompts > 0, "everything capture includes an actual provider prompt");
}
/// A failed live run's trace in words, before it is erased: its prompts and each failure's
/// class, status, code, field, phase, state and exit. No content, detail or evidence.
pub fn failures(directory: &Path) -> String {
    let Ok(bytes) = fs::read(directory.join("agent-trace.jsonl")) else {
        return "live agent trace: absent".to_owned();
    };
    let (mut prompts, mut failures) = (0, Vec::new());
    for line in bytes.split(|byte| *byte == b'\n').filter(|line| !line.is_empty()) {
        let Ok(record) = serde_json::from_slice::<serde_json::Value>(line) else { continue };
        if record.get("prompt").is_some_and(serde_json::Value::is_object) {
            prompts += 1;
        }
        failed(&record, &mut failures);
    }
    format!("live agent trace: prompts={prompts}; failures={}", serde_json::Value::Array(failures))
}
fn failed(value: &serde_json::Value, failures: &mut Vec<serde_json::Value>) {
    match value {
        serde_json::Value::Object(fields) => {
            for (name, value) in fields {
                if name == "failure" && value.is_object() {
                    let named = ["class", "status", "code", "field", "phase", "state", "exit"]
                        .into_iter()
                        .filter_map(|name| value.get(name).map(|value| (name.to_owned(), value.clone())))
                        .collect();
                    failures.push(serde_json::Value::Object(named));
                } else {
                    failed(value, failures);
                }
            }
        }
        serde_json::Value::Array(values) => values.iter().for_each(|value| failed(value, failures)),
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::String(_) => {}
    }
}
fn fields(value: &serde_json::Value, credential: Option<&[u8]>) {
    match value {
        serde_json::Value::Object(fields) => {
            for (name, value) in fields {
                assert!(
                    !["authorization", "access_token", "refresh_token", "id_token", "credential", "grant"]
                        .contains(&name.to_ascii_lowercase().as_str()),
                    "credential field in Smith trace"
                );
                self::fields(value, credential);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                fields(value, credential);
            }
        }
        serde_json::Value::String(text) => {
            assert!(!text.to_ascii_lowercase().contains("authorization:"), "Authorization header in Smith trace");
            if let Some(credential) = credential {
                assert_ne!(text.as_bytes(), credential, "grant value in Smith trace");
            }
        }
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {}
    }
}
