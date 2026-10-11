//! Scan source names and comments while leaving quoted data with its owner.

use std::path::Path;

fn identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte >= 128
}

fn forbidden_name(name: &str) -> bool {
    let stem = name.trim_end_matches(|ch: char| ch.is_ascii_digit());
    if stem.len() < name.len() && (stem.ends_with('V') || stem.ends_with("_v")) {
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

fn vocabulary_at(path: &Path, source: &str, at: usize, words: &[String], failures: &mut Vec<String>) {
    let bytes = source.as_bytes();
    for word in words {
        if !bytes[at].eq_ignore_ascii_case(&word.as_bytes()[0]) {
            continue;
        }
        let end = at + word.len();
        if bytes.get(at..end).is_some_and(|candidate| candidate.eq_ignore_ascii_case(word.as_bytes()))
            && !bytes.get(end).is_some_and(|after| after.is_ascii_alphanumeric() && !after.is_ascii_uppercase())
        {
            failures.push(format!("{}: {word}", path.display()));
        }
    }
}

pub(super) fn breaches(path: &Path, source: &str, _lower: &str) -> Vec<String> {
    scan(path, source, &[], true)
}

// One source walk checks every vocabulary pattern and each lexeme's name rule.
pub(super) fn scan(path: &Path, source: &str, words: &[String], names: bool) -> Vec<String> {
    let mut failures = Vec::new();
    let mut starts = [false; 256];
    for word in words {
        starts[usize::from(word.as_bytes()[0].to_ascii_lowercase())] = true;
        starts[usize::from(word.as_bytes()[0].to_ascii_uppercase())] = true;
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
            let mut candidate = false;
            while bytes.get(index).is_some_and(|byte| identifier_byte(*byte)) {
                candidate |= matches!(bytes[index], b't' | b'T')
                    && bytes.get(index..index + 5).is_some_and(|word| word.eq_ignore_ascii_case(b"typed"));
                candidate |= matches!(bytes[index], b'l' | b'L')
                    && bytes.get(index..index + 6).is_some_and(|word| word.eq_ignore_ascii_case(b"legacy"));
                candidate |= bytes[index] == b'V' && bytes.get(index + 1).is_some_and(u8::is_ascii_digit);
                candidate |=
                    bytes.get(index..index + 2) == Some(b"_v") && bytes.get(index + 2).is_some_and(u8::is_ascii_digit);
                index += 1;
            }
            let name = &source[start..index];
            if candidate && forbidden_name(name) {
                failures.push(format!("{}:{line}: identifier {name}", path.display()));
            }
        } else {
            index += source[index..].chars().next().expect("source character").len_utf8();
            while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
                index += 1;
            }
        }
        for (offset, byte) in bytes[start..index].iter().copied().enumerate() {
            let at = start + offset;
            if starts[usize::from(byte)] && (at == 0 || !bytes[at - 1].is_ascii_alphanumeric()) {
                vocabulary_at(path, source, at, words, &mut failures);
            }
            if comment
                && matches!(byte, b'l' | b'L')
                && bytes.get(at..at + 6).is_some_and(|word| word.eq_ignore_ascii_case(b"legacy"))
                && !at.checked_sub(1).and_then(|before| bytes.get(before)).is_some_and(|byte| identifier_byte(*byte))
                && !bytes.get(at + 6).is_some_and(|byte| identifier_byte(*byte))
            {
                let word = &source[at..at + 6];
                failures.push(format!("{}:{line}: comment {word}", path.display()));
            }
            if byte == b'\n' {
                line += 1;
            }
        }
    }
    failures
}
