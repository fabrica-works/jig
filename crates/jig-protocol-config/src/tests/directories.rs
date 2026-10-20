use super::fixtures;
use crate::{Directories, Directory, Engine, Path, Problem, Problems, Segment, Transport, What, Worker};

// configuration.md, 10: both a path and a given directory are at most 4096 bytes.
const BOUND: usize = 4096;
const PAST: usize = 4097;
fn given() -> Directories {
    Directories::given(b"/c", b"/d/", b"/k").unwrap()
}
fn resolved(directory: Directory, path: &[u8], expected: &[u8]) {
    assert_eq!(given().join(directory, path).unwrap().as_ref(), expected);
}

#[test]
fn a_path_resolves_by_its_kind() {
    table([
        ("absolute", || resolved(Directory::Data, b"/a/../b", b"/a/../b")),
        ("config", || resolved(Directory::Config, b"x", b"/c/x")),
        ("data trailing slash", || resolved(Directory::Data, b"x", b"/d/x")),
        ("cache", || resolved(Directory::Cache, b"x", b"/k/x")),
        ("dot", || resolved(Directory::Config, b"./x", b"/c/./x")),
        ("empty component", || resolved(Directory::Config, b"a//b", b"/c/a//b")),
        ("dots prefix", || resolved(Directory::Config, b"..x", b"/c/..x")),
        ("dots suffix", || resolved(Directory::Config, b"x..", b"/c/x..")),
        ("working", || {
            assert_eq!(Directories::working().join(Directory::Data, b"x").unwrap().as_ref(), b"x");
        }),
        ("root", || {
            let directories = Directories::given(b"/", b"/", b"/").unwrap();
            assert_eq!(directories.join(Directory::Config, b"x").unwrap().as_ref(), b"/x");
        }),
    ]);
}
fn relative_refused(directories: Directories) {
    for (label, path) in [
        ("empty", b"".as_slice()),
        ("parent", b".."),
        ("parent prefix", b"../x"),
        ("parent inside", b"a/../b"),
        ("parent suffix", b"a/.."),
    ] {
        assert_eq!(directories.join(Directory::Config, path), Err(What::Relative), "{label}");
    }
}
#[test]
fn a_relative_path_is_refused() {
    table([("given", || relative_refused(given())), ("working", || relative_refused(Directories::working()))]);
}
#[test]
fn a_joined_path_past_the_bound_is_refused() {
    table([
        ("exactly the bound", || {
            // /c/ consumes three bytes of the 4096-byte result.
            let path = [b'x'; BOUND - 3];
            let expected = joined(&[b"/c/", &path]);
            assert_eq!(given().join(Directory::Config, &path).unwrap().as_ref(), expected.as_ref());
        }),
        ("one past", || {
            let path = [b'x'; PAST - 3];
            assert_eq!(given().join(Directory::Config, &path), Err(What::Bound));
        }),
        ("working at bound", || {
            let path = [b'x'; BOUND];
            assert_eq!(Directories::working().join(Directory::Data, &path).unwrap().as_ref(), path);
        }),
        ("working past bound", || {
            assert_eq!(Directories::working().join(Directory::Data, &[b'x'; PAST]), Err(What::Bound));
        }),
        ("absolute at bound", || {
            let mut path = [b'x'; BOUND];
            *path.first_mut().unwrap() = b'/';
            assert_eq!(given().join(Directory::Cache, &path).unwrap().as_ref(), path);
        }),
        ("absolute past bound", || {
            let mut path = [b'x'; PAST];
            *path.first_mut().unwrap() = b'/';
            assert_eq!(given().join(Directory::Cache, &path), Err(What::Bound));
        }),
    ]);
}
fn directory_refused(directory: &[u8]) {
    for (label, directories) in [
        ("config", Directories::given(directory, b"/d", b"/k")),
        ("data", Directories::given(b"/c", directory, b"/k")),
        ("cache", Directories::given(b"/c", b"/d", directory)),
    ] {
        assert_eq!(directories, None, "{label}");
    }
}
#[test]
fn given_directories_are_absolute_and_bounded() {
    table([
        ("relative", || directory_refused(b"relative")),
        ("empty", || directory_refused(b"")),
        ("too long", || {
            let mut path = [b'x'; PAST];
            *path.first_mut().unwrap() = b'/';
            directory_refused(&path);
        }),
        ("at bound in every position", || {
            let mut path = [b'x'; BOUND];
            *path.first_mut().unwrap() = b'/';
            assert!(Directories::given(&path, &path, &path).is_some());
        }),
    ]);
}

const TLS: &[u8] = br#""transport":"tls","server_name":"issuer.example","trust":"certs/endpoint""#;
const PROVIDERS: &[u8] = br#""providers":[{"name":"Fake","signed_in_plan":true,"manage_usage":"https://usage.example/","number":1,"models":["small"],"sources":{"sign_in":{"issuer":"https://issuer.example","client":"client-a","scopes":["chat"],"authorization":"https://issuer.example/authorize","token":"https://issuer.example/token","keys":"https://issuer.example/keys","transport":"tls","server_name":"issuer.example","trust":"certs/registration","must_grant":["chat"],"redirect_path":"callback"}}}],"endpoints":"#;
fn engine_bytes() -> Box<[u8]> {
    let bytes = fixtures::with(fixtures::ENGINE_SMALL, br#""transport": "plaintext""#, TLS);
    let bytes =
        fixtures::with(&bytes, br#""providers": [{"name":"Fake","number":1,"models":["small"],"sources":{}}],"#, b"");
    fixtures::with(&bytes, br#""endpoints":"#, PROVIDERS)
}
fn document(bytes: &[u8]) -> skein_json::Document {
    let limits = crate::DocumentLimits { bytes: 8192, tokens: 1536, depth: 16 };
    crate::document(bytes, &limits).unwrap()
}
fn engine(bytes: &[u8]) -> Result<Engine, Problems> {
    crate::engine(&document(bytes), 0, &given())
}
fn worker(bytes: &[u8]) -> Result<Worker, Problems> {
    crate::worker(&document(bytes), 0, &given())
}
fn item(names: &[&[u8]], index: u32) -> Path {
    fixtures::path(names).child(Segment::Index(index))
}
fn trust(transport: &Transport) -> &[u8] {
    match transport {
        Transport::Tls { trust, .. } => trust.as_deref().unwrap(),
        Transport::Plaintext => panic!("TLS fixture"),
    }
}
fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut len = 0_usize;
    for part in parts {
        len = len.checked_add(part.len()).unwrap();
    }
    let mut writer = skein_lib::Writer::new(len);
    for part in parts {
        writer.put(part).unwrap();
    }
    writer.finish()
}
fn named_file<'a>(files: &'a [crate::File], at: &Path) -> &'a crate::File {
    for file in files {
        if &file.at == at {
            return file;
        }
    }
    panic!("expected configured file {at:?}")
}
fn file_paths(files: &[crate::File], expected: &[(Path, &[u8])]) {
    assert_eq!(files.len(), expected.len());
    for (at, path) in expected {
        let file = named_file(files, at);
        assert_eq!(file.path.as_ref(), *path, "{at:?}");
    }
}
fn engine_paths() {
    let engine = engine(&engine_bytes()).unwrap();
    let registration = engine.llm.providers.first().unwrap().sign_in.as_ref().unwrap();
    let rows: &[(&str, &[u8], &[u8])] = &[
        ("deployment", &engine.deployment.directory, b"/d/state"),
        ("instructions", &engine.charters.first().unwrap().policy.instructions, b"/c/charters/chat.txt"),
        ("worker a digest", &engine.hosts.workers.first().unwrap().digest, b"/d/secrets/worker-a.digest"),
        ("worker b digest", &engine.hosts.workers.get(1).unwrap().digest, b"/d/secrets/worker-b.digest"),
        ("endpoint trust", trust(&engine.llm.endpoints.first().unwrap().transport), b"/c/certs/endpoint"),
        ("registration trust", trust(&registration.endpoint_transport), b"/c/certs/registration"),
    ];
    for (label, value, expected) in rows {
        assert_eq!(value, expected, "{label}");
    }
    file_paths(
        &crate::files(&engine),
        &[
            (item(&[b"charters"], 0).member(b"instructions"), b"/c/charters/chat.txt"),
            (item(&[b"hosts", b"workers"], 0).member(b"digest"), b"/d/secrets/worker-a.digest"),
            (item(&[b"hosts", b"workers"], 1).member(b"digest"), b"/d/secrets/worker-b.digest"),
            (item(&[b"llm", b"endpoints"], 0).member(b"trust"), b"/c/certs/endpoint"),
            (
                item(&[b"llm", b"providers"], 0).member(b"sources").member(b"sign_in").member(b"trust"),
                b"/c/certs/registration",
            ),
        ],
    );
}
fn worker_paths() {
    let bytes = fixtures::with(fixtures::WORKER_SMALL, br#""agent"]"#, br#""agent", "charters/x", "../x"]"#);
    let bytes = fixtures::with(&bytes, b"jig-test", b"../program");
    let worker = worker(&bytes).unwrap();
    for (label, value, expected) in [
        ("secret", worker.secret.as_ref(), b"/d/secrets/worker-a.secret".as_slice()),
        ("configuration", worker.agent.configuration.as_ref(), b"/c/agent.json"),
        ("program", worker.agent.program.as_ref(), b"../program"),
        ("argument", worker.agent.arguments.get(1).unwrap().as_ref(), b"charters/x"),
        ("parent argument", worker.agent.arguments.get(2).unwrap().as_ref(), b"../x"),
    ] {
        assert_eq!(value, expected, "{label}");
    }
    file_paths(
        &crate::worker_files(&worker),
        &[
            (fixtures::path(&[b"secret"]), b"/d/secrets/worker-a.secret"),
            (fixtures::path(&[b"agent", b"configuration"]), b"/c/agent.json"),
        ],
    );
}
#[test]
fn each_path_member_is_joined_to_its_kind() {
    table([("engine", engine_paths), ("worker", worker_paths)]);
}
fn engine_refusals() {
    let bytes = engine_bytes();
    let rows: &[(&str, &[u8], Path)] = &[
        ("deployment", b"state", fixtures::path(&[b"deployment", b"directory"])),
        ("instructions", b"charters/chat.txt", item(&[b"charters"], 0).member(b"instructions")),
        ("digest", b"secrets/worker-a.digest", item(&[b"hosts", b"workers"], 0).member(b"digest")),
        ("endpoint trust", b"certs/endpoint", item(&[b"llm", b"endpoints"], 0).member(b"trust")),
        (
            "registration trust",
            b"certs/registration",
            item(&[b"llm", b"providers"], 0).member(b"sources").member(b"sign_in").member(b"trust"),
        ),
    ];
    for (label, from, path) in rows {
        let changed = fixtures::with(&bytes, from, b"../x");
        let problems = engine(&changed).unwrap_err();
        assert_eq!(problems.problems.get(0), Some(&Problem { path: path.clone(), what: What::Relative }), "{label}");
    }
}
fn worker_refusals() {
    for (label, from, path) in [
        ("secret", b"secrets/worker-a.secret".as_slice(), fixtures::path(&[b"secret"])),
        ("configuration", b"agent.json", fixtures::path(&[b"agent", b"configuration"])),
    ] {
        let bytes = fixtures::with(fixtures::WORKER_SMALL, from, b"../x");
        let problems = worker(&bytes).unwrap_err();
        assert_eq!(problems.problems.get(0), Some(&Problem { path, what: What::Relative }), "{label}");
    }
}
#[test]
fn a_relative_member_is_refused_naming_it() {
    table([("engine", engine_refusals), ("worker", worker_refusals)]);
}
#[test]
fn an_absolute_member_is_taken_as_written() {
    table([
        ("config", || resolved(Directory::Config, b"/absolute/../x", b"/absolute/../x")),
        ("data", || resolved(Directory::Data, b"/absolute/../x", b"/absolute/../x")),
        ("cache", || resolved(Directory::Cache, b"/absolute/../x", b"/absolute/../x")),
    ]);
}

type Case = (&'static str, fn());
fn table<const N: usize>(cases: [Case; N]) {
    let mut failures = [None; N];
    let mut passed = true;
    for ((name, case), failure) in cases.into_iter().zip(&mut failures) {
        if std::panic::catch_unwind(case).is_err() {
            *failure = Some(name);
            passed = false;
        }
    }
    assert!(passed, "{failures:?}");
}
