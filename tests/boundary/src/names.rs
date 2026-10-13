//! Scan source names and comments while leaving quoted data with its owner.

use std::path::Path;

fn identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte >= 128
}

fn codec_module(name: &str) -> bool {
    let Some((family, version)) = name.rsplit_once("_v") else {
        return false;
    };
    super::tests::kit_root()
        .join("crates/jig-protocol-store/schema")
        .join(format!("{family}-v{version}.schema"))
        .is_file()
}

fn forbidden_name(name: &str, codec: bool) -> bool {
    let stem = name.trim_end_matches(|ch: char| ch.is_ascii_digit());
    if !codec && stem.len() < name.len() && (stem.ends_with('V') || stem.ends_with("_v")) {
        return true;
    }
    let bytes = name.as_bytes();
    let mut start = 0;
    for end in 0..=bytes.len() {
        let boundary = end == bytes.len()
            || bytes[end] == b'_'
            || (end > start
                && bytes[end].is_ascii_uppercase()
                && (bytes[end - 1].is_ascii_lowercase() || bytes.get(end + 1).is_some_and(u8::is_ascii_lowercase)))
            || (end > start && bytes[end].is_ascii_digit());
        if boundary {
            let part = &name[start..end];
            if matches!(part.len(), 5..=7)
                && ["Typed", "Untyped", "Legacy"].iter().any(|word| part.eq_ignore_ascii_case(word))
            {
                return true;
            }
            start = if bytes.get(end) == Some(&b'_') { end + 1 } else { end };
        }
    }
    false
}

fn quoted_end(source: &[u8], quote: usize) -> usize {
    let mut end = quote + 1;
    while end < source.len() {
        match source[end] {
            b'\\' => end += 2,
            b'"' => return end + 1,
            _ => end += 1,
        }
    }
    source.len()
}

fn literal_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut quote = start;
    if bytes[quote] == b'b' || bytes[quote] == b'c' {
        quote += 1;
    }
    if bytes.get(quote) == Some(&b'r') {
        let mut body = quote + 1;
        while bytes.get(body) == Some(&b'#') {
            body += 1;
        }
        if bytes.get(body) != Some(&b'"') {
            return None;
        }
        let hashes = body - quote - 1;
        for (offset, _) in source[body + 1..].match_indices('"') {
            let end = body + 2 + offset;
            if bytes.get(end..end + hashes).is_some_and(|tail| tail.iter().all(|byte| *byte == b'#')) {
                return Some(end + hashes);
            }
        }
        return Some(bytes.len());
    }
    match bytes.get(quote) {
        Some(b'"') => Some(quoted_end(bytes, quote)),
        Some(b'\'') => {
            let mut end = quote + 1;
            let ch = source.get(end..)?.chars().next()?;
            end += ch.len_utf8();
            if ch == '\\' {
                if bytes.get(end) == Some(&b'u') {
                    end = source[end..].find('}').map(|offset| end + offset + 1)?;
                } else if bytes.get(end) == Some(&b'x') {
                    end += 3;
                } else {
                    end += 1;
                }
            }
            (bytes.get(end) == Some(&b'\'')).then_some(end + 1)
        }
        _ => None,
    }
}

fn comment_end(source: &[u8], start: usize) -> usize {
    if source[start + 1] == b'/' {
        return source[start..].iter().position(|byte| *byte == b'\n').map_or(source.len(), |end| start + end);
    }
    let mut depth = 1;
    let mut end = start + 2;
    while end < source.len() && depth != 0 {
        match source.get(end..end + 2) {
            Some(b"/*") => {
                depth += 1;
                end += 2;
            }
            Some(b"*/") => {
                depth -= 1;
                end += 2;
            }
            _ => end += 1,
        }
    }
    end
}

// Foreign boundary fields are allowed only at their declared construction site.
fn foreign_fields(path: &Path, source: &str) -> Vec<usize> {
    if !path.ends_with("testing/jig-test-worker/src/tests/fixtures.rs") {
        return Vec::new();
    }
    let bytes = source.as_bytes();
    let mut fields = Vec::new();
    let mut index = 0;
    let mut depth = 0_usize;
    let mut directories = Vec::new();
    while index < bytes.len() {
        if let Some(end) = literal_end(source, index) {
            index = end;
            continue;
        }
        if matches!(bytes.get(index..index + 2), Some(b"//" | b"/*")) {
            index = comment_end(bytes, index);
            continue;
        }
        let named = "smith::Directory";
        if source[index..].starts_with(named)
            && !index
                .checked_sub(1)
                .and_then(|before| bytes.get(before))
                .is_some_and(|byte| identifier_byte(*byte) || *byte == b':')
        {
            let mut brace = index + named.len();
            while bytes.get(brace).is_some_and(u8::is_ascii_whitespace) {
                brace += 1;
            }
            if bytes.get(brace) == Some(&b'{') {
                directories.push(depth + 1);
                index = brace;
            }
        }
        match bytes[index] {
            b'{' => depth += 1,
            b'}' => {
                if directories.last() == Some(&depth) {
                    directories.pop();
                }
                depth = depth.saturating_sub(1);
            }
            _ => {
                let field = [b'g', b'i', b't'];
                if directories.last() == Some(&depth)
                    && bytes.get(index..index + field.len()) == Some(&field)
                    && !index
                        .checked_sub(1)
                        .and_then(|previous| bytes.get(previous))
                        .is_some_and(|byte| identifier_byte(*byte))
                {
                    let mut colon = index + field.len();
                    while bytes.get(colon).is_some_and(u8::is_ascii_whitespace) {
                        colon += 1;
                    }
                    if bytes.get(colon) == Some(&b':') {
                        fields.push(index);
                    }
                }
            }
        }
        index += source[index..].chars().next().expect("source character").len_utf8();
    }
    fields
}

fn vocabulary_matches(path: &Path, source: &str, words: &[String]) -> Vec<String> {
    let lower = source.to_ascii_lowercase();
    let bytes = source.as_bytes();
    let mut matches = Vec::new();
    let foreign = foreign_fields(path, source);
    for (word_index, word) in words.iter().enumerate() {
        for (at, _) in lower.match_indices(word.as_str()) {
            if word.as_bytes() == [b'g', b'i', b't'] && foreign.contains(&at) {
                continue;
            }
            if !at.checked_sub(1).and_then(|before| bytes.get(before)).is_some_and(u8::is_ascii_alphanumeric)
                && !bytes
                    .get(at + word.len())
                    .is_some_and(|after| after.is_ascii_alphanumeric() && !after.is_ascii_uppercase())
            {
                matches.push((at, word_index));
            }
        }
    }
    matches.sort_unstable();
    matches.into_iter().map(|(_, word)| format!("{}: {}", path.display(), words[word])).collect()
}

pub(super) fn breaches(path: &Path, source: &str) -> Vec<String> {
    scan(path, source, &[], true)
}

// Standard string searches check vocabulary; the source walk checks lexeme names.
pub(super) fn scan(path: &Path, source: &str, words: &[String], names: bool) -> Vec<String> {
    let mut failures = vocabulary_matches(path, source, words);
    if !names {
        return failures;
    }
    let bytes = source.as_bytes();
    let mut index = 0;
    let mut line = 1;
    while index < bytes.len() {
        let start = index;
        let literal = if names && matches!(bytes[index], b'b' | b'c' | b'r' | b'"' | b'\'') {
            literal_end(source, index)
        } else {
            None
        };
        let mut comment = false;
        if let Some(end) = literal {
            index = end;
        } else if names && matches!(bytes.get(index..index + 2), Some(b"//" | b"/*")) {
            index = comment_end(bytes, index);
            comment = true;
        } else if names && identifier_byte(bytes[index]) && !bytes[index].is_ascii_digit() {
            while bytes.get(index).is_some_and(|byte| identifier_byte(*byte)) {
                index += 1;
            }
            let name = &source[start..index];
            let foreign_ip = matches!(name, "V4" | "V6")
                && source[..start].strip_suffix("core::net::IpAddr::").is_some_and(|prefix| {
                    prefix.chars().next_back().is_none_or(|ch| !(ch.is_alphanumeric() || ch == '_' || ch == ':'))
                });
            if !foreign_ip && forbidden_name(name, false) && (!codec_module(name) || forbidden_name(name, true)) {
                failures.push(format!("{}:{line}: identifier {name}", path.display()));
            }
        } else {
            index += source[index..].chars().next().expect("source character").len_utf8();
            while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
                index += 1;
            }
        }
        if comment {
            for (offset, _) in source[start..index].to_ascii_lowercase().match_indices("legacy") {
                let at = start + offset;
                if !at.checked_sub(1).and_then(|before| bytes.get(before)).is_some_and(|byte| identifier_byte(*byte))
                    && !bytes.get(at + 6).is_some_and(|byte| identifier_byte(*byte))
                {
                    let comment_line = line + source[start..at].matches('\n').count();
                    let word = &source[at..at + 6];
                    failures.push(format!("{}:{comment_line}: comment {word}", path.display()));
                }
            }
        }
        line += source[start..index].matches('\n').count();
    }
    failures
}

#[test]
fn only_the_foreign_directory_field_is_allowed_at_its_named_site() {
    let field = ["g", "it"].concat();
    let site = Path::new("testing/jig-test-worker/src/tests/fixtures.rs");
    let other = Path::new("testing/other/src/fixtures.rs");
    let literal = format!("fn setup() {{ let directory = smith::Directory {{ {field}: false }}; }}");
    let cases = [
        ("foreign field", site, literal.clone(), 0),
        ("elsewhere in named file", site, format!("{literal} fn {field}() {{}}"), 1),
        ("other file", other, literal, 1),
        ("other record", site, format!("let directory = other::Directory {{ {field}: false }};"), 1),
        ("other qualifier", site, format!("let directory = other_smith::Directory {{ {field}: false }};"), 1),
        (
            "nested record",
            site,
            format!("let directory = smith::Directory {{ nested: Other {{ {field}: false }} }};"),
            1,
        ),
        ("quoted prefix", site, format!("let text = \"smith::Directory {{ {field}: false }}\";"), 1),
    ];
    for (case, path, source, count) in cases {
        let actual = vocabulary_matches(path, &source, core::slice::from_ref(&field));
        let expected = vec![format!("{}: {field}", path.display()); count];
        assert_eq!(actual, expected, "{case}");
    }
}
