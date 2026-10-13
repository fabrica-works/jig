//! One borrowed account, a real agent completion, and the native person's answer.
mod support;
use jig_client_protocol_world::native::Script;
use jig_protocol_config as config;
use jig_protocol_store::{codec, transcripts};
use skein_lib::Duration;
use skein_shell::Clock;
use std::{fs, path::Path};

const INSTRUCTIONS: &str = "Report exactly: Process answer";
const WORDS: &str = "Follow the first sentence.";

fn configuration(directory: &Path, source: &Path) -> codec::Limits {
    support::configuration(directory);
    let settings: serde_json::Value =
        serde_json::from_slice(&fs::read(source).expect("live settings")).expect("public live settings JSON");
    let path = directory.join("config.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("fixture configuration")).expect("fixture JSON");
    let engine = &mut value["engine"];
    engine["llm"]["accounts"][0]["source"]["handed_in"]["file"] = settings["account_file"].clone();
    let endpoint = &mut engine["llm"]["endpoints"][0];
    endpoint["provider"] = "codex".into();
    endpoint["address"] = serde_json::json!({"socket":settings["address"]});
    endpoint["transport"] = "tls".into();
    endpoint["server_name"] = "chatgpt.com".into();
    let model = &mut engine["llm"]["models"][0];
    model["name"] = settings["model"].clone();
    model["output"] = 4096.into();
    engine["charters"][0]["model"] = settings["model"].clone();
    engine["charters"][0]["turns"] = 5.into();
    engine["charters"][0]["time_ms"] = 240_000.into();
    engine["charters"][0]["waiting_ms"] = 10_000.into();
    let bytes = serde_json::to_vec(&value).expect("live deployment JSON");
    fs::write(&path, &bytes).expect("live deployment configuration");
    let document = config::document(&bytes, &config::DocumentLimits { bytes: 1 << 20, tokens: 1 << 20, depth: 32 })
        .expect("live document");
    let at = config::read::Object::new(&document, 0)
        .expect("document object")
        .member(&document, b"engine")
        .expect("engine member");
    let engine = config::engine(&document, at).expect("live engine configuration");
    let limits = codec::limits(&engine.limits.core).expect("live store codec limits");
    let path = directory.join("agent.json");
    let mut agent: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("agent fixture")).expect("agent JSON");
    let endpoint = &mut agent["endpoints"][0];
    endpoint["address"] = settings["address"].clone();
    endpoint["server_name"] = "chatgpt.com".into();
    endpoint.as_object_mut().expect("endpoint").remove("trust_der");
    endpoint["connect"] = 10_000.into();
    endpoint["handshake"] = 10_000.into();
    endpoint["reasoning_effort"] = "low".into();
    endpoint["headers"] = serde_json::json!(skein_llm::openai::identity::headers().iter().map(|header|
        serde_json::json!({"name":String::from_utf8_lossy(&header.name),"value":String::from_utf8_lossy(&header.value)})
    ).collect::<Vec<_>>());
    endpoint["models"][0]["name"] = settings["model"].clone();
    endpoint["models"][0]["output"] = 4096.into();
    endpoint["models"][0]["reasoning_item"] = 2048.into();
    agent["trace"] =
        serde_json::json!({"path":directory.join("agent-trace.jsonl"),"capture":"everything","delivery":"complete"});
    fs::write(path, serde_json::to_vec(&agent).expect("live agent JSON")).expect("live agent configuration");
    fs::write(directory.join("charters/chat.txt"), INSTRUCTIONS.as_bytes()).expect("live charter");
    limits
}

fn completions(directory: &Path, limits: &codec::Limits) -> u32 {
    let path = directory.join("state/transcripts/0000000000000000/0000000000000001");
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return 0,
        Err(error) => panic!("live transcript: {error}"),
    };
    let mut at = 0;
    let mut count = 0;
    while at + 4 <= bytes.len() {
        let length = u32::from_be_bytes(bytes[at..at + 4].try_into().expect("frame length")) as usize;
        if length < 12 || at + length > bytes.len() {
            break;
        }
        let turn = transcripts::frame::decode(&bytes[at..at + length], limits).expect("durable live turn");
        assert_eq!((turn.task, turn.attempt, turn.turn), (1, 1, count + 1));
        // Smith's transcript.md, 1: a turn is one completion, including compaction.
        count += 1;
        at += length;
    }
    count
}

fn capture(directory: &Path, destination: &Path) -> bool {
    let bytes = fs::read(directory.join("agent-trace.jsonl")).expect("Smith provider trace");
    let mut requests = Vec::new();
    for line in bytes.split(|byte| *byte == b'\n').filter(|line| !line.is_empty()) {
        let record: serde_json::Value = serde_json::from_slice(line).expect("Smith event JSON");
        if let Some(prompt) = record.get("prompt") {
            requests.push(serde_json::json!({"instructions":prompt["system"],"input":prompt["messages"]}));
        }
    }
    fs::write(destination, serde_json::to_vec_pretty(&requests).expect("request capture JSON"))
        .expect("system/input capture file");
    let present = requests.iter().any(|request| {
        request["instructions"].as_str().is_some_and(|system| system.contains(INSTRUCTIONS) && system.contains(WORDS))
    });
    eprintln!(
        "live request capture={}; brief={}; file={}",
        requests.len(),
        if present { "present" } else { "absent" },
        destination.display()
    );
    assert!(!requests.is_empty(), "provider request captured before interpreting the answer");
    present
}

#[test]
fn a_native_client_s_chat_is_answered_by_a_real_model() {
    let source = std::env::var_os("JIG_LIVE_CONFIG").expect("the live proof script supplies configuration");
    let directory = support::scratch();
    let limits = configuration(directory.path(), Path::new(&source));
    let clock = Clock::new();
    let started = clock.now().now;
    let result = std::panic::catch_unwind(|| {
        support::run(
            directory.path(),
            Script::answered_report(WORDS.as_bytes(), support::ANSWER, Duration::from_secs(240)),
            None,
        )
    });
    let directory = directory.keep();
    let count = completions(&directory, &limits);
    let wall_ms = clock.now().now.saturating_since(started).as_nanos() / 1_000_000;
    eprintln!("live completions={count}; wall={wall_ms}ms; state={}", directory.display());
    let captured = capture(&directory, &Path::new(&source).with_file_name("live-request.json"));
    let outcome = result.unwrap_or_else(|failed| {
        eprintln!("live failed state retained at {}", directory.display());
        std::panic::resume_unwind(failed)
    });
    assert!(captured, "the provider request includes charter and task words");
    assert!((1..=5).contains(&count), "at most five real completions, and at least one answer");
    assert_eq!(outcome, (1, 1));
}
