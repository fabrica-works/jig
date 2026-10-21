//! The committed client codec and bytes against its schema (skein's codec.md, 6).
use skein_codegen::{generate, parse};
use std::{fs, path::Path};

#[test]
fn client_wire_schema_matches_generated_code_and_goldens() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/jig-client-protocol-wire");
    let mut source = fs::read_to_string(root.join("schema/client-v12.schema")).expect("client schema");
    source.push_str(
        &fs::read_to_string(root.join("schema/client-v12-part1.schema")).expect("client schema continuation"),
    );
    check(&root, &source, "v12");
}
#[test]
fn view_records_schema_matches_generated_code_and_goldens() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/jig-client-view");
    let source = fs::read_to_string(root.join("schema/records-v1.schema")).expect("view records schema");
    check(&root, &source, "records-v1");
}
#[test]
fn ops_client_wire_schema_matches_generated_code_and_goldens() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ops/client-protocol-wire");
    let source = fs::read_to_string(root.join("schema/ops-client-v1.schema")).expect("ops client schema");
    check(&root, &source, "v1");
}
fn check(root: &Path, source: &str, version: &str) {
    let output = generate(&parse(source).expect("schema valid"));
    let file = match version {
        "v12" => "src/generated/v12.rs",
        "v1" => "src/generated/v1.rs",
        _ => "src/generated/records_v1.rs",
    };
    let code = root.join(file);
    let goldens = root.join("golden").join(version);
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    let again = "run SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world --test drift";
    if write {
        fs::create_dir_all(code.parent().expect("generated code directory")).expect("create code directory");
        fs::write(&code, &output.rust).expect("write generated code");
        fs::create_dir_all(&goldens).expect("create golden directory");
    } else {
        assert_eq!(fs::read_to_string(&code).expect("generated code committed"), output.rust, "{again}");
    }
    for golden in output.goldens {
        let path = goldens.join(golden.name);
        if write {
            fs::write(&path, golden.bytes).expect("write golden bytes");
        } else {
            assert_eq!(fs::read(&path).expect("golden bytes committed"), golden.bytes, "{again}");
        }
    }
}
