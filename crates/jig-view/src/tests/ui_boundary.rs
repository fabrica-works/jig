//! The generic UI can move whole without taking an owner with it.

const SOURCES: &[&str] = &[
    "ui.rs",
    "ui/binding.rs",
    "ui/builder.rs",
    "ui/diff.rs",
    "ui/limits.rs",
    "ui/markdown.rs",
    "ui/outline.rs",
    "ui/phrases.rs",
    "ui/tests/binding.rs",
    "ui/tests/diff.rs",
    "ui/tests/fixtures.rs",
    "ui/tests/markdown/01.md",
    "ui/tests/markdown/02.md",
    "ui/tests/markdown/03.md",
    "ui/tests/markdown/04.md",
    "ui/tests/markdown/05.md",
    "ui/tests/markdown/06.md",
    "ui/tests/markdown/07.md",
    "ui/tests/markdown/08.md",
    "ui/tests/markdown/09.md",
    "ui/tests/markdown/10.md",
    "ui/tests/markdown/11.md",
    "ui/tests/markdown/12.md",
    "ui/tests/markdown.rs",
    "ui/tests/outline.rs",
    "ui/tests/phrases.rs",
    "ui/tests/slots.rs",
    "ui/tests/tree.rs",
    "ui/tests.rs",
    "ui/tree.rs",
    "ui/view.rs",
];

#[test]
fn the_ui_module_names_only_itself_and_skein_lib() {
    for file in SOURCES {
        let path = std::path::Path::new("src").join(file);
        let source = std::fs::read_to_string(path).expect("every listed UI source file exists");
        for (index, line) in source.lines().enumerate() {
            for rest in line.split("crate::").skip(1) {
                assert!(rest.starts_with("ui::"), "{file}:{} names outside the UI: {line}", index + 1);
            }
            assert!(!line.contains("jig_domain_client"), "{file}:{} names the domain: {line}", index + 1);
            assert!(!line.contains("jig_"), "{file}:{} names an owner: {line}", index + 1);
        }
    }
}
