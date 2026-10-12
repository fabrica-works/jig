use super::graph::{all_crates, breaches};
use super::parse_manifest;
use super::{forbidden_words, kit_root, source_files, workspace_manifest, workspace_vocabulary};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn vocabulary_breaches(path: &Path, source: &str, _lower: &str, words: &[String]) -> Vec<String> {
    super::super::names::scan(path, source, words, false)
}

fn kit_vocabulary(part: usize) {
    let mut files = Vec::new();
    source_files(&kit_root(), true, &[".rs", "Cargo.toml"], &mut files);
    files.sort();
    let words = forbidden_words();
    let failures: Vec<_> = files
        .iter()
        .enumerate()
        .filter(|(index, _)| index % 64 == part)
        .flat_map(|(_, path)| {
            let source = fs::read_to_string(path).expect("source is readable");
            let source = if path.file_name().is_some_and(|name| name == "Cargo.toml") && workspace_manifest(&source) {
                workspace_vocabulary(&source)
            } else {
                source
            };
            super::super::names::scan(
                path,
                &source,
                &words,
                path.extension().is_some_and(|extension| extension == "rs"),
            )
        })
        .collect();
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
fn kit_sources_part_17_keep_the_kit_vocabulary() {
    kit_vocabulary(16);
}

#[test]
fn kit_sources_part_18_keep_the_kit_vocabulary() {
    kit_vocabulary(17);
}

#[test]
fn kit_sources_part_19_keep_the_kit_vocabulary() {
    kit_vocabulary(18);
}

#[test]
fn kit_sources_part_20_keep_the_kit_vocabulary() {
    kit_vocabulary(19);
}

#[test]
fn kit_sources_part_21_keep_the_kit_vocabulary() {
    kit_vocabulary(20);
}

#[test]
fn kit_sources_part_22_keep_the_kit_vocabulary() {
    kit_vocabulary(21);
}

#[test]
fn kit_sources_part_23_keep_the_kit_vocabulary() {
    kit_vocabulary(22);
}

#[test]
fn kit_sources_part_24_keep_the_kit_vocabulary() {
    kit_vocabulary(23);
}

#[test]
fn kit_sources_part_25_keep_the_kit_vocabulary() {
    kit_vocabulary(24);
}

#[test]
fn kit_sources_part_26_keep_the_kit_vocabulary() {
    kit_vocabulary(25);
}

#[test]
fn kit_sources_part_27_keep_the_kit_vocabulary() {
    kit_vocabulary(26);
}

#[test]
fn kit_sources_part_28_keep_the_kit_vocabulary() {
    kit_vocabulary(27);
}

#[test]
fn kit_sources_part_29_keep_the_kit_vocabulary() {
    kit_vocabulary(28);
}

#[test]
fn kit_sources_part_30_keep_the_kit_vocabulary() {
    kit_vocabulary(29);
}

#[test]
fn kit_sources_part_31_keep_the_kit_vocabulary() {
    kit_vocabulary(30);
}

#[test]
fn kit_sources_part_32_keep_the_kit_vocabulary() {
    kit_vocabulary(31);
}

#[test]
fn kit_sources_part_33_keep_the_kit_vocabulary() {
    kit_vocabulary(32);
}

#[test]
fn kit_sources_part_34_keep_the_kit_vocabulary() {
    kit_vocabulary(33);
}

#[test]
fn kit_sources_part_35_keep_the_kit_vocabulary() {
    kit_vocabulary(34);
}

#[test]
fn kit_sources_part_36_keep_the_kit_vocabulary() {
    kit_vocabulary(35);
}

#[test]
fn kit_sources_part_37_keep_the_kit_vocabulary() {
    kit_vocabulary(36);
}

#[test]
fn kit_sources_part_38_keep_the_kit_vocabulary() {
    kit_vocabulary(37);
}

#[test]
fn kit_sources_part_39_keep_the_kit_vocabulary() {
    kit_vocabulary(38);
}

#[test]
fn kit_sources_part_40_keep_the_kit_vocabulary() {
    kit_vocabulary(39);
}

#[test]
fn kit_sources_part_41_keep_the_kit_vocabulary() {
    kit_vocabulary(40);
}

#[test]
fn kit_sources_part_42_keep_the_kit_vocabulary() {
    kit_vocabulary(41);
}

#[test]
fn kit_sources_part_43_keep_the_kit_vocabulary() {
    kit_vocabulary(42);
}

#[test]
fn kit_sources_part_44_keep_the_kit_vocabulary() {
    kit_vocabulary(43);
}

#[test]
fn kit_sources_part_45_keep_the_kit_vocabulary() {
    kit_vocabulary(44);
}

#[test]
fn kit_sources_part_46_keep_the_kit_vocabulary() {
    kit_vocabulary(45);
}

#[test]
fn kit_sources_part_47_keep_the_kit_vocabulary() {
    kit_vocabulary(46);
}

#[test]
fn kit_sources_part_48_keep_the_kit_vocabulary() {
    kit_vocabulary(47);
}

#[test]
fn kit_sources_part_49_keep_the_kit_vocabulary() {
    kit_vocabulary(48);
}

#[test]
fn kit_sources_part_50_keep_the_kit_vocabulary() {
    kit_vocabulary(49);
}

#[test]
fn kit_sources_part_51_keep_the_kit_vocabulary() {
    kit_vocabulary(50);
}

#[test]
fn kit_sources_part_52_keep_the_kit_vocabulary() {
    kit_vocabulary(51);
}

#[test]
fn kit_sources_part_53_keep_the_kit_vocabulary() {
    kit_vocabulary(52);
}

#[test]
fn kit_sources_part_54_keep_the_kit_vocabulary() {
    kit_vocabulary(53);
}

#[test]
fn kit_sources_part_55_keep_the_kit_vocabulary() {
    kit_vocabulary(54);
}

#[test]
fn kit_sources_part_56_keep_the_kit_vocabulary() {
    kit_vocabulary(55);
}

#[test]
fn kit_sources_part_57_keep_the_kit_vocabulary() {
    kit_vocabulary(56);
}

#[test]
fn kit_sources_part_58_keep_the_kit_vocabulary() {
    kit_vocabulary(57);
}

#[test]
fn kit_sources_part_59_keep_the_kit_vocabulary() {
    kit_vocabulary(58);
}

#[test]
fn kit_sources_part_60_keep_the_kit_vocabulary() {
    kit_vocabulary(59);
}

#[test]
fn kit_sources_part_61_keep_the_kit_vocabulary() {
    kit_vocabulary(60);
}

#[test]
fn kit_sources_part_62_keep_the_kit_vocabulary() {
    kit_vocabulary(61);
}

#[test]
fn kit_sources_part_63_keep_the_kit_vocabulary() {
    kit_vocabulary(62);
}

#[test]
fn kit_sources_part_64_keep_the_kit_vocabulary() {
    kit_vocabulary(63);
}

#[test]
fn workspace_fixture_omits_only_dependency_sources() {
    let word = ["g", "it"].concat();
    let url = format!("https://{word}.ekanayaka.io/ai/skein.{word}");
    let manifest = format!("[workspace]\n[workspace.dependencies]\nskein-lib = {{ {word} = \"{url}\" }}\n");
    let source = workspace_vocabulary(&manifest);
    assert_eq!(source, "[workspace]\n[workspace.dependencies]\nskein-lib = {  }\n");
    assert!(
        vocabulary_breaches(Path::new("fixture/Cargo.toml"), &source, &source.to_ascii_lowercase(), &forbidden_words())
            .is_empty()
    );

    let commented = format!("{manifest}# {word} = \"{url}\"\n");
    let source = workspace_vocabulary(&commented);
    assert!(source.ends_with(&format!("# {word} = \"{url}\"\n")));
    assert_eq!(
        vocabulary_breaches(Path::new("fixture/Cargo.toml"), &source, &source.to_ascii_lowercase(), &forbidden_words())
            .len(),
        3
    );

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
    let failures =
        vocabulary_breaches(Path::new("fixture/src/lib.rs"), &source, &source.to_ascii_lowercase(), &forbidden_words());
    assert_eq!(failures, [format!("fixture/src/lib.rs: {word}")]);
}

#[test]
fn name_fixture_reports_each_file_line_and_breach() {
    let source = include_str!("../../fixtures/names.txt");
    let failures = super::super::names::breaches(Path::new("fixture/names.rs"), source, &source.to_ascii_lowercase());
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
