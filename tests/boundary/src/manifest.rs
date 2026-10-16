//! The kit manifests and their normal and target dependencies.

use std::{
    fs,
    path::{Path, PathBuf},
};

/// A package name and its normal and target dependencies; the workspace has no name.
#[derive(Debug)]
pub struct CrateManifest {
    pub path: PathBuf,
    pub name: Option<String>,
    pub dependencies: Vec<String>,
}

pub(crate) fn walk_directory(name: &str, at_root: bool) -> bool {
    !(name.starts_with('.') || name == "golden" || (at_root && name.starts_with("target")))
}

pub(crate) fn source_files(dir: &Path, at_root: bool, suffixes: &[&str], out: &mut Vec<PathBuf>) {
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

pub(crate) fn quoted_value(value: &str) -> Option<&str> {
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

/// Read a package name and its normal and target dependencies.
#[must_use]
pub fn parse_manifest(path: PathBuf, source: &str) -> CrateManifest {
    let mut section = String::new();
    let mut name = None;
    let mut dependencies = Vec::new();
    for line in source.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            line.trim_matches(['[', ']']).clone_into(&mut section);
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

pub(crate) fn workspace_manifest(source: &str) -> bool {
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

/// Read manifests under `root`, skipping hidden, golden and root target directories.
#[must_use]
pub fn manifests(root: &Path) -> Vec<CrateManifest> {
    let mut files = Vec::new();
    source_files(root, true, &["Cargo.toml"], &mut files);
    files
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(&path).expect("manifest is readable");
            parse_manifest(path, &source)
        })
        .collect()
}
