//! The committed client codec and bytes against its schema (skein's codec.md, 6).
use skein_codegen::{generate, parse};
use std::{fs, path::Path};

#[test]
fn client_wire_schema_matches_generated_code_and_goldens() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/jig-protocol-client-wire");
    let source = fs::read_to_string(root.join("schema/client-v2.schema")).expect("client schema");
    let output = generate(&parse(&source).expect("schema valid"));
    let code = root.join("src/generated/v2.rs");
    let goldens = root.join("golden/v2");
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    let again = "run SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world --test drift";
    if write {
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
