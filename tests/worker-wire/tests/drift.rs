use skein_codegen::{generate, parse};
use std::{fs, path::Path};

#[test]
fn worker_wire_schema_matches_generated_code_and_goldens() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/jig-protocol-worker-wire");
    let source = fs::read_to_string(root.join("schema/worker-wire-v1.schema")).expect("schema");
    let schema = parse(&source).expect("schema valid");
    let output = generate(&schema);
    let committed = fs::read_to_string(root.join("src/generated/v1.rs")).expect("generated code");
    assert_eq!(committed, output.rust, "regenerate with skein-codegen");
    for golden in output.goldens {
        let committed = fs::read(root.join("golden/v1").join(&golden.name)).expect("golden bytes");
        assert_eq!(committed, golden.bytes, "golden {} drifted", golden.name);
    }
}
