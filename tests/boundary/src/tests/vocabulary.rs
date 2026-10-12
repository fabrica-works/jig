use super::graph::{all_crates, breaches};
use super::parse_manifest;
use super::{forbidden_words, kit_root, source_files, workspace_manifest, workspace_vocabulary};
use std::{
    fs,
    path::{Path, PathBuf},
};

const PARTS: usize = 16;

fn vocabulary_breaches(path: &Path, source: &str, words: &[String]) -> Vec<String> {
    super::super::names::scan(path, source, words, false)
}

fn vocabulary_files(root: &Path) -> Vec<(PathBuf, u64)> {
    let mut files = Vec::new();
    source_files(root, true, &[".rs", "Cargo.toml", ".schema"], &mut files);
    files.retain(|path| {
        !path.ancestors().any(|parent| {
            parent.file_name().is_some_and(|name| name == "generated")
                && parent.parent().is_some_and(|parent| parent.file_name().is_some_and(|name| name == "src"))
        }) && (path.extension().is_none_or(|extension| extension != "schema")
            || path.parent().is_some_and(|parent| parent.file_name().is_some_and(|name| name == "schema")))
    });
    files.sort();
    files
        .into_iter()
        .map(|path| {
            let bytes = fs::metadata(&path).expect("source has metadata").len();
            (path, bytes)
        })
        .collect()
}

fn vocabulary_part(files: &[(PathBuf, u64)], part: usize, words: &[String]) -> Vec<String> {
    let total = files.iter().map(|(_, bytes)| bytes).sum::<u64>().max(1);
    let mut before = 0;
    let mut failures = Vec::new();
    for (path, bytes) in files {
        let file_part = (before * u64::try_from(PARTS).expect("part count fits") / total)
            .min(u64::try_from(PARTS - 1).expect("last part fits"));
        before += bytes;
        if file_part != u64::try_from(part).expect("part fits") {
            continue;
        }
        let source = fs::read_to_string(path).expect("source is readable");
        let source = if path.file_name().is_some_and(|name| name == "Cargo.toml") && workspace_manifest(&source) {
            workspace_vocabulary(&source)
        } else {
            source
        };
        failures.extend(super::super::names::scan(
            path,
            &source,
            words,
            path.extension().is_some_and(|extension| extension == "rs" || extension == "schema"),
        ));
    }
    failures
}

fn kit_vocabulary(part: usize) {
    let failures = vocabulary_part(&vocabulary_files(&kit_root()), part, &forbidden_words());
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn kit_sources_keep_the_kit_vocabulary() {
    kit_vocabulary(0);
}

#[test]
fn kit_sources_part_02_keep_the_kit_vocabulary() {
    kit_vocabulary(1);
}

#[test]
fn kit_sources_part_03_keep_the_kit_vocabulary() {
    kit_vocabulary(2);
}

#[test]
fn kit_sources_part_04_keep_the_kit_vocabulary() {
    kit_vocabulary(3);
}

#[test]
fn kit_sources_part_05_keep_the_kit_vocabulary() {
    kit_vocabulary(4);
}

#[test]
fn kit_sources_part_06_keep_the_kit_vocabulary() {
    kit_vocabulary(5);
}

#[test]
fn kit_sources_part_07_keep_the_kit_vocabulary() {
    kit_vocabulary(6);
}

#[test]
fn kit_sources_part_08_keep_the_kit_vocabulary() {
    kit_vocabulary(7);
}

#[test]
fn kit_sources_part_09_keep_the_kit_vocabulary() {
    kit_vocabulary(8);
}

#[test]
fn kit_sources_part_10_keep_the_kit_vocabulary() {
    kit_vocabulary(9);
}

#[test]
fn kit_sources_part_11_keep_the_kit_vocabulary() {
    kit_vocabulary(10);
}

#[test]
fn kit_sources_part_12_keep_the_kit_vocabulary() {
    kit_vocabulary(11);
}

#[test]
fn kit_sources_part_13_keep_the_kit_vocabulary() {
    kit_vocabulary(12);
}

#[test]
fn kit_sources_part_14_keep_the_kit_vocabulary() {
    kit_vocabulary(13);
}

#[test]
fn kit_sources_part_15_keep_the_kit_vocabulary() {
    kit_vocabulary(14);
}

#[test]
fn kit_sources_part_16_keep_the_kit_vocabulary() {
    kit_vocabulary(15);
}

#[test]
fn workspace_fixture_omits_only_dependency_sources() {
    let word = ["g", "it"].concat();
    let url = format!("https://{word}.ekanayaka.io/ai/skein.{word}");
    let manifest = format!("[workspace]\n[workspace.dependencies]\nskein-lib = {{ {word} = \"{url}\" }}\n");
    let source = workspace_vocabulary(&manifest);
    assert_eq!(source, "[workspace]\n[workspace.dependencies]\nskein-lib = {  }\n");
    assert!(vocabulary_breaches(Path::new("fixture/Cargo.toml"), &source, &forbidden_words()).is_empty());

    let commented = format!("{manifest}# {word} = \"{url}\"\n");
    let source = workspace_vocabulary(&commented);
    assert!(source.ends_with(&format!("# {word} = \"{url}\"\n")));
    assert_eq!(vocabulary_breaches(Path::new("fixture/Cargo.toml"), &source, &forbidden_words()).len(), 3);

    let other = format!("[workspace]\nlabel = '{word}'\n{word} = true\n");
    assert_eq!(workspace_vocabulary(&other), other);
    let quoted = format!("[workspace]\n[workspace.dependencies.skein-lib]\n\"{word}\" = '{url}'\nversion = \"1\"\n");
    assert_eq!(workspace_vocabulary(&quoted), "[workspace]\n[workspace.dependencies.skein-lib]\n\nversion = \"1\"\n");
}

#[test]
fn fixtures_name_the_path_and_the_breach() {
    let forbidden = ["tem", "per"].concat();
    let manifest =
        format!("[package]\nname = \"jig-sample\"\n[dependencies]\n{forbidden} = {{ path = \"../example\" }}\n");
    let parsed = parse_manifest(PathBuf::from("fixture/Cargo.toml"), &manifest);
    let failures = breaches(&[parsed], all_crates);
    assert_eq!(failures, [format!("fixture/Cargo.toml: {forbidden}")]);

    let workspace_manifest =
        format!("[package]\nname = \"jig-sample\"\n[dependencies]\n{forbidden}.workspace = true\n");
    let parsed = parse_manifest(PathBuf::from("fixture/workspace/Cargo.toml"), &workspace_manifest);
    let failures = breaches(&[parsed], all_crates);
    assert_eq!(failures, [format!("fixture/workspace/Cargo.toml: {forbidden}")]);

    let word = ["for", "ge"].concat();
    let source = format!("const SUBJECT: &str = \"{word}\";");
    let failures = vocabulary_breaches(Path::new("fixture/src/lib.rs"), &source, &forbidden_words());
    assert_eq!(failures, [format!("fixture/src/lib.rs: {word}")]);
}

#[test]
fn name_fixture_reports_each_file_line_and_breach() {
    let source = include_str!("../../fixtures/names.txt");
    let failures = super::super::names::breaches(Path::new("fixture/names.rs"), source);
    assert_eq!(
        failures,
        [
            "fixture/names.rs:1: identifier AnswerV2",
            "fixture/names.rs:2: identifier finish_v19",
            "fixture/names.rs:3: identifier TypedCall",
            "fixture/names.rs:4: identifier UntypedCall",
            "fixture/names.rs:5: identifier LegacyCall",
            "fixture/names.rs:6: identifier assign_typed",
            "fixture/names.rs:7: identifier legacy_call",
            "fixture/names.rs:8: comment LEGACY",
            "fixture/names.rs:10: comment legacy",
        ]
    );
}

#[test]
fn one_source_walk_checks_all_patterns_in_quotes_comments_and_name_boundaries() {
    use std::fmt::Write as _;

    let words = forbidden_words();
    let mut source = String::new();
    for word in &words {
        write!(source, " {word} {word}Upper x{word} {word}lower _{word} é{word} ").expect("fixture string");
        write!(source, "\n// {word}\nconst SUBJECT: &str = r#\"{word}\"#;\n").expect("fixture string");
    }
    let path = Path::new("fixture/patterns.rs");
    let lower = source.to_ascii_lowercase();
    let mut expected = Vec::new();
    for word in &words {
        for (at, _) in lower.match_indices(word) {
            let before = source[..at].chars().next_back();
            let after = source[at + word.len()..].chars().next();
            let word_char = |ch: char| ch.is_ascii_alphanumeric();
            if !before.is_some_and(word_char)
                && (!after.is_some_and(word_char) || after.is_some_and(char::is_uppercase))
            {
                expected.push(format!("{}: {word}", path.display()));
            }
        }
    }
    expected.sort();
    for names in [false, true] {
        let mut actual = super::super::names::scan(path, &source, &words, names);
        actual.sort();
        assert_eq!(actual, expected, "all patterns retain the independent boundary definition");
    }
}

struct VocabularyFixture(PathBuf);

impl Drop for VocabularyFixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("fixture is removed");
    }
}

fn vocabulary_fixture(word: &str) -> VocabularyFixture {
    let root = std::env::temp_dir().join(format!("jig-vocabulary-{}", std::process::id()));
    let fixture = VocabularyFixture(root);
    let family = fixture.0.join("crates/sample");
    for directory in ["schema", "src/generated"] {
        fs::create_dir_all(family.join(directory)).expect("fixture directory is created");
    }
    fs::write(family.join("schema/example.schema"), format!("record {word} {{}}")).expect("fixture schema is written");
    fs::write(family.join("src/lib.rs"), format!("{} {word}", " ".repeat(256))).expect("fixture source is written");
    fs::write(family.join("src/generated/example.rs"), word).expect("fixture generated source is written");
    fixture
}

#[test]
fn vocabulary_fixture_reports_schema_and_source_past_a_byte_split() {
    let word = ["for", "ge"].concat();
    let fixture = vocabulary_fixture(&word);
    let files = vocabulary_files(&fixture.0);
    assert_eq!(files.len(), 2, "generated sources are replaced by their schema");
    let total = files.iter().map(|(_, bytes)| bytes).sum::<u64>();
    assert!(total / u64::try_from(PARTS).expect("part count fits") < 256, "the word follows a byte split");
    let words = forbidden_words();
    let failures: Vec<_> = (0..PARTS).flat_map(|part| vocabulary_part(&files, part, &words)).collect();
    assert_eq!(
        failures,
        [
            format!("{}: {word}", fixture.0.join("crates/sample/schema/example.schema").display()),
            format!("{}: {word}", fixture.0.join("crates/sample/src/lib.rs").display()),
        ]
    );
}
