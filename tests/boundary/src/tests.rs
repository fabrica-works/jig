mod graph;
mod vocabulary;

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct CrateManifest {
    path: PathBuf,
    name: Option<String>,
    dependencies: Vec<String>,
}

fn kit_root() -> PathBuf {
    // Cargo and nextest run tests from the current package directory. A cached
    // binary may have been compiled in a worktree that has since been removed.
    std::env::current_dir().expect("package directory exists").join("../..").canonicalize().expect("kit root exists")
}

fn walk_directory(name: &str, at_root: bool) -> bool {
    !(name.starts_with('.') || (at_root && name == "target"))
}

fn source_files(dir: &Path, at_root: bool, suffixes: &[&str], out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("kit directory is readable") {
        let entry = entry.expect("directory entry is readable");
        let kind = entry.file_type().expect("directory entry has a file type");
        let path = entry.path();
        if kind.is_dir() || (kind.is_symlink() && path.is_dir()) {
            if path.file_name().and_then(|name| name.to_str()).is_some_and(|name| walk_directory(name, at_root)) {
                source_files(&path, false, suffixes, out);
            }
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| suffixes.iter().any(|suffix| name.ends_with(suffix)))
        {
            out.push(path);
        }
    }
}

fn quoted_value(value: &str) -> Option<&str> {
    let value = value.trim();
    let quote = value.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    value[1..].split_once(quote).map(|(name, _)| name)
}

fn dependency_name(line: &str) -> Option<String> {
    let (key, value) = line.split_once('=')?;
    let key = key.trim().split_once('.').map_or(key.trim(), |(name, _)| name);
    let key = key.trim_matches('"').trim_matches('\'');
    if key.is_empty() {
        return None;
    }
    let package =
        value.split_once("package").and_then(|(_, rest)| rest.split_once('=')).and_then(|(_, rest)| quoted_value(rest));
    Some(package.unwrap_or(key).to_owned())
}

fn parse_manifest(path: PathBuf, source: &str) -> CrateManifest {
    let mut section = String::new();
    let mut name = None;
    let mut dependencies = Vec::new();
    for line in source.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            section = line.trim_matches(['[', ']']).to_owned();
        } else if section == "package" && line.starts_with("name =") {
            name = Some(
                line.split_once('=')
                    .and_then(|(_, value)| quoted_value(value))
                    .expect("package name is quoted")
                    .to_owned(),
            );
        } else if (section == "dependencies" || section.ends_with(".dependencies"))
            && !line.starts_with('#')
            && let Some(dependency) = dependency_name(line)
        {
            dependencies.push(dependency);
        }
    }
    assert!(
        name.as_ref().is_some_and(|name| !name.is_empty()) || workspace_manifest(source),
        "{} has no package name",
        path.display()
    );
    CrateManifest { path, name, dependencies }
}

fn workspace_manifest(source: &str) -> bool {
    let mut workspace = false;
    for line in source.lines().map(str::trim) {
        let section = line.split('#').next().expect("line has a first part").trim();
        if section == "[package]" {
            return false;
        }
        workspace |= section == "[workspace]";
    }
    workspace
}

fn quoted_end(source: &str, start: usize) -> usize {
    let bytes = source.as_bytes();
    let quote = bytes[start];
    let mut end = start + 1;
    while end < bytes.len() {
        if bytes[end] == quote {
            return end + 1;
        }
        if quote == b'"' && bytes[end] == b'\\' {
            end += 1;
        }
        end += 1;
    }
    bytes.len()
}

fn dependency_source_end(source: &str, key_end: usize) -> Option<usize> {
    let tail = source[key_end..].trim_start_matches([' ', '\t']);
    let value = tail.strip_prefix('=')?.trim_start_matches([' ', '\t']);
    if !value.starts_with(['"', '\'']) {
        return None;
    }
    let start = source.len() - value.len();
    Some(quoted_end(source, start))
}

fn workspace_vocabulary(source: &str) -> String {
    let key = ["g", "it"].concat();
    let mut vocabulary = String::new();
    let mut start = 0;
    while start < source.len() {
        let tail = &source[start..];
        let ch = tail.chars().next().expect("source has remaining text");
        let end = if ch == '#' {
            tail.find('\n').map_or(source.len(), |end| start + end)
        } else if ch == '"' || ch == '\'' {
            quoted_end(source, start)
        } else if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            start + tail.find(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-').unwrap_or(tail.len())
        } else {
            start + ch.len_utf8()
        };
        let token = &source[start..end];
        let is_key = token == key || quoted_value(token) == Some(key.as_str());
        if is_key && let Some(pair_end) = dependency_source_end(source, end) {
            start = pair_end;
        } else {
            vocabulary.push_str(token);
            start = end;
        }
    }
    vocabulary
}

fn manifests() -> Vec<CrateManifest> {
    let mut files = Vec::new();
    source_files(&kit_root(), true, &["Cargo.toml"], &mut files);
    files
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(&path).expect("manifest is readable");
            parse_manifest(path, &source)
        })
        .collect()
}

fn forbidden_words() -> Vec<String> {
    [
        vec!["tem", "per"],
        vec!["for", "ge"],
        vec!["for", "gejo"],
        vec!["repo", "sitory"],
        vec!["repo", "sitories"],
        vec!["bra", "nch"],
        vec!["bra", "nches"],
        vec!["pull", " request"],
        vec!["g", "it"],
        vec!["wi", "ki"],
    ]
    .into_iter()
    .map(|parts| parts.concat())
    .collect()
}

#[test]
fn workspace_fixture_checks_dependencies_without_a_package() {
    let forbidden = ["tem", "per", "-sample"].concat();
    let manifest =
        format!("[workspace]\nmembers = []\n[workspace.dependencies]\n{forbidden} = {{ path = \"../example\" }}\n");
    let parsed = parse_manifest(PathBuf::from("fixture/Cargo.toml"), &manifest);
    assert_eq!(parsed.name, None);
    assert_eq!(graph::breaches(&[parsed], graph::all_crates), [format!("fixture/Cargo.toml: {forbidden}")]);
    assert!(!workspace_manifest("[workspace]\n[package]\nname = \"jig-sample\"\n"));
}

#[test]
fn walk_fixture_skips_build_and_hidden_directories() {
    assert!(!walk_directory("target", true));
    assert!(!walk_directory(&[".", "g", "it"].concat(), true));
    assert!(!walk_directory(".config", false));
    assert!(walk_directory("tests", true));
    assert!(walk_directory("target", false));
}
