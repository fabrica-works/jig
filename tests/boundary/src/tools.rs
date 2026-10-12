//! Host tools carry the core's vocabulary and link no agent or application.
const MANIFEST: &str = include_str!("../../../crates/jig-protocol-tools/Cargo.toml");

#[test]
fn core_tools_link_no_smith() {
    for line in MANIFEST.lines() {
        assert!(!line.trim().starts_with("smith-"), "{line}");
    }
}

#[test]
fn core_tools_link_no_application() {
    const OWNERS: &[&str] = &["jig-domain-core", "jig-domain-tasks", "jig-domain-notes", "jig-domain-authority"];
    for line in MANIFEST.lines() {
        let Some((name, _)) = line.split_once('=') else { continue };
        let key = name.trim();
        let name = key.strip_suffix(".workspace").unwrap_or(key);
        if name.starts_with("jig-") {
            assert!(OWNERS.contains(&name), "{line}");
        }
    }
}
