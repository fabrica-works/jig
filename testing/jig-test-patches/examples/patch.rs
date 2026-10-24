//! Test-only driver of skein-codegen's patch command (codec.md 12.8).
//! heavy cargo run -p jig-test-patches --example patch -- <crate> <base> [<after-level>|baseline]
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use skein_codegen::patch::{self, Landed, Options};

fn git(owner: &Path, args: &[&str]) -> Vec<u8> {
    let output = Command::new("git").arg("-C").arg(owner).args(args).output().expect("git runs");
    assert!(output.status.success(), "git: {}", String::from_utf8_lossy(&output.stderr));
    output.stdout
}

#[expect(clippy::print_stdout, reason = "the test-only command prints codegen's report")]
fn main() {
    let mut args = std::env::args().skip(1);
    let owner = PathBuf::from(args.next().expect("crate path"));
    let base = args.next().expect("base commit");
    let mode = args.next().expect("baseline or after level");
    assert!(args.next().is_none(), "three arguments");
    let paths = git(&owner, &["ls-tree", "-r", "--name-only", "-z", &base, "--", "patches", "schema/kept"]);
    let mut files = BTreeMap::new();
    for bytes in paths.split(|byte| *byte == 0).filter(|path| !path.is_empty()) {
        let name = core::str::from_utf8(bytes).expect("UTF-8 path");
        let path = PathBuf::from(name);
        assert!(path.starts_with("patches") || path.starts_with("schema/kept"), "crate paths only");
        let object = format!("{base}:./{name}");
        let contents = git(&owner, &["show", &object]);
        assert!(files.insert(path, contents).is_none(), "unique path");
    }
    let fixture = owner.ends_with("jig-test-patches");
    let options = Options {
        owner: Some(if fixture { "fixture" } else { "fixture-jig" }.into()),
        baseline: mode == "baseline",
        after: if fixture && mode != "baseline" {
            vec![("fixture-jig".into(), mode.parse().expect("upstream level"))]
        } else { vec![] },
        ..Options::default()
    };
    let report = patch::run(&owner, &Landed::new(files), &options).expect("patch command runs");
    print!("{report}");
    assert!(report.findings.is_empty(), "resolve codegen's findings");
}
