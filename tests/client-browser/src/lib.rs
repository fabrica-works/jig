//! The browser tier starts the real test service with a private small deployment.
use skein_fake_person_browser::{
    Setup,
    harness::{Names, Output, Printed},
};
use std::{
    fs,
    net::TcpListener,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
mod forgery;

#[must_use]
pub fn jig_test() -> PathBuf {
    let exe = std::env::current_exe().expect("test executable");
    let binary = exe.parent().and_then(Path::parent).expect("target directory").join("jig-test");
    assert!(binary.is_file(), "build the browser server with heavy cargo build -p jig-test: {}", binary.display());
    binary
}
#[must_use]
pub fn chromium() -> PathBuf {
    if let Some(path) = std::env::var_os("SKEIN_TEST_CHROMIUM") {
        let path = PathBuf::from(path);
        assert!(path.is_file(), "SKEIN_TEST_CHROMIUM must name a file");
        return path.canonicalize().expect("configured Chromium");
    }
    for directory in std::env::split_paths(&std::env::var_os("PATH").expect("PATH")) {
        let path = directory.join("chromium");
        if path.is_file() {
            return path.canonicalize().expect("Chromium on PATH");
        }
    }
    panic!("Chromium is required: set SKEIN_TEST_CHROMIUM or put chromium on PATH");
}
#[must_use]
pub fn bundle() -> PathBuf {
    let path = PathBuf::from(
        std::env::var_os("JIG_TEST_BUNDLE").expect("run scripts/check.sh to build the bundle and set JIG_TEST_BUNDLE"),
    );
    let path = if path.is_absolute() { path } else { Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(path) };
    assert!(path.join("index.html").is_file(), "run scripts/check.sh to build the bundle at {}", path.display());
    path.canonicalize().expect("absolute bundle")
}
fn private(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).expect("fixture file");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private fixture");
}
fn configuration(directory: &Path, bundle: &Path) -> serde_json::Value {
    let mut value: serde_json::Value =
        serde_json::from_slice(include_bytes!("configuration.json")).expect("engine JSON");
    value["service"]["components"] = serde_json::json!(["clients"]);
    value["workers"] = serde_json::json!([]);
    value["application"]["server"] = serde_json::json!({"listener":{"socket":"127.0.0.1:0"},"bundle":bundle});
    for pointer in [
        "/engine/rules/ceiling/delegation/kinds",
        "/engine/projects/0/policy/ceiling/delegation/kinds",
        "/engine/projects/0/policy/roles/0/authority/delegation/kinds",
    ] {
        value
            .pointer_mut(pointer)
            .expect("delegation kinds")
            .as_array_mut()
            .expect("kinds array")
            .push(serde_json::json!({"kind":"role", "number":0}));
    }
    absolute_paths(&mut value, directory);
    value
}
fn absolute_paths(value: &mut serde_json::Value, directory: &Path) {
    match value {
        serde_json::Value::String(text)
            if ["state", "charters/chat.txt", "secrets/worker-a.digest", "accounts/chat.json", "app.secret"]
                .contains(&text.as_str()) =>
        {
            directory.join(&*text).to_str().expect("fixture path").clone_into(text);
        }
        serde_json::Value::Array(items) => {
            for item in items {
                absolute_paths(item, directory);
            }
        }
        serde_json::Value::Object(items) => {
            for item in items.values_mut() {
                absolute_paths(item, directory);
            }
        }
        _ => {}
    }
}
const FILES: &[(&str, &[u8])] = &[
    ("charters/chat.txt", b"Reply to the person."),
    ("secrets/worker-a.digest", b"6fb46f7a92742970166379ed5195e79c4493a7cc5664280c039cfd4095ba5faf"),
    ("accounts/chat.json", br#"{"access_token":"test-token","expires_at":1767229200,"account_id":"acc"}"#),
    ("app.secret", b"application-token"),
];
pub struct Tier {
    scratch: skein_scratch::Scratch,
    forged: Box<[u8]>,
}
impl Tier {
    #[must_use]
    pub fn new(name: &str) -> Self {
        let scratch = skein_scratch::Scratch::new(name);
        for name in ["accounts", "accounts/records", "secrets", "charters", "state"] {
            fs::create_dir(scratch.path().join(name)).expect("deployment directory");
        }
        for (name, bytes) in FILES {
            private(&scratch.path().join(name), bytes);
        }
        let value = configuration(scratch.path(), &bundle());
        private(&scratch.path().join("config.json"), &serde_json::to_vec(&value).expect("deployment JSON"));
        Self { scratch, forged: forgery::request() }
    }
    #[must_use]
    pub fn target(&self) -> jig_client_fake_person_browser::Target<'_> {
        jig_client_fake_person_browser::Target {
            prefix: b"/jig",
            page: b"/",
            request: &self.forged,
            done: skein_fake_person::Find::text(b"Forged"),
        }
    }
    #[must_use]
    pub fn reserved(name: &str) -> Self {
        let tier = Self::new(name);
        let listener = TcpListener::bind("127.0.0.1:0").expect("reserve restart port");
        let address = listener.local_addr().expect("restart address");
        let path = tier.scratch.path().join("config.json");
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("deployment")).expect("JSON");
        value["application"]["server"]["listener"]["socket"] = serde_json::json!(address.to_string());
        private(&path, &serde_json::to_vec(&value).expect("reserved deployment JSON"));
        tier
    }
    #[must_use]
    pub fn request_script(&self) -> skein_fake_person_browser::Script {
        use skein_fake_person::{Find, Step};
        use skein_fake_person_browser::{Script, Step as BrowserStep};
        use skein_ui::Role;
        Script::new(Box::new([
            BrowserStep::Person(Step::Press { find: Find::named(Role::Link, b"Conversations") }),
            BrowserStep::Person(Step::See {
                find: Find::named(Role::TextArea, b"First words"),
                within: skein_lib::Duration::from_secs(5),
            }),
            BrowserStep::Person(Step::Type {
                find: Find::named(Role::TextArea, b"First words"),
                words: b"Reload story".as_slice().into(),
            }),
            BrowserStep::Person(Step::Press { find: Find::named(Role::Button, b"Start a conversation") }),
        ]))
    }
    #[must_use]
    pub fn setup(&self) -> Setup<'_> {
        let args = vec!["serve".into(), self.scratch.path().join("config.json").into_os_string()];
        Setup {
            server: jig_test(),
            args: args.clone(),
            restart: args,
            printed: Printed { prefix: b"http://127.0.0.1:", on: Output::Stdout, names: Names::Url },
            chromium: chromium(),
            expected: vec![],
            origins: vec![],
        }
    }
    #[must_use]
    pub fn states(&self) -> jig_client_fake_person_browser::States {
        use jig_client_view::phrases::{english, jig::JigPhrase};
        use skein_fake_person::Find;
        const PHRASE_WORDS: u32 = 32;
        let book = english::english(PHRASE_WORDS).expect("English");
        let find = |phrase: JigPhrase| Find::text(&book.say(0, phrase.id(), &[]).expect("state phrase").text);
        jig_client_fake_person_browser::States {
            live: find(JigPhrase::Live),
            offline: find(JigPhrase::Offline),
            signed_out: find(JigPhrase::SignInPage),
        }
    }
    #[must_use]
    pub fn words(&self, name: &[u8]) -> Box<[u8]> {
        assert_eq!(name, b"inbox");
        jig_client_view::phrases::english::english(32)
            .expect("English")
            .say(0, jig_client_view::phrases::jig::JigPhrase::Inbox.id(), &[])
            .expect("inbox phrase")
            .text
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn tier_configuration() -> serde_json::Value {
        configuration(Path::new("/private/test"), Path::new("/test/bundle"))
    }
    #[test]
    fn the_tier_names_the_bundle_for_its_server() {
        assert_eq!(tier_configuration()["application"]["server"]["bundle"], "/test/bundle");
    }
    #[test]
    fn the_tier_listens_on_loopback_at_a_dynamic_port() {
        assert_eq!(tier_configuration()["application"]["server"]["listener"]["socket"], "127.0.0.1:0");
    }
    #[test]
    fn the_tier_starts_only_the_clients_service_component() {
        assert_eq!(tier_configuration()["service"]["components"], serde_json::json!(["clients"]));
    }
    #[test]
    fn the_tier_uses_the_engine_s_small_limits_profile() {
        assert_eq!(tier_configuration()["engine"]["limits"]["profile"], "small");
    }
    #[test]
    fn the_tier_starts_without_any_agent_workers() {
        assert_eq!(tier_configuration()["workers"], serde_json::json!([]));
    }
}
