//! Project names' shape and Unicode simple case folding (domain/core.md, 6.1).
mod fold;

/// Why a project's name is out of shape (core.md, 6.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NameRule {
    Empty,
    Long,
    Control,
    Space,
    Digits,
}

/// The most bytes a project's name has.
pub const NAME_BYTES: usize = 64;

/// The rule `name` breaks, or None when it is in shape.
#[must_use]
pub fn name_rule(name: &str) -> Option<NameRule> {
    if name.is_empty() {
        return Some(NameRule::Empty);
    }
    if name.len() > NAME_BYTES {
        return Some(NameRule::Long);
    }
    for char in name.chars() {
        if char.is_control() {
            return Some(NameRule::Control);
        }
    }
    for edge in [name.chars().next(), name.chars().next_back()] {
        if let Some(char) = edge
            && char.is_whitespace()
        {
            return Some(NameRule::Space);
        }
    }
    let mut digits = true;
    for char in name.chars() {
        digits &= char.is_ascii_digit();
    }
    if digits { Some(NameRule::Digits) } else { None }
}

/// Whether two names are the same under Unicode simple case folding.
#[must_use]
pub fn same_name(a: &str, b: &str) -> bool {
    if a.chars().count() != b.chars().count() {
        return false;
    }
    for (a, b) in a.chars().zip(b.chars()) {
        if folded(a) != folded(b) {
            return false;
        }
    }
    true
}

fn folded(char: char) -> char {
    let mut low = 0_usize;
    let mut high = fold::FOLD.len();
    for _ in 0..usize::BITS {
        if low == high {
            break;
        }
        let half = high.checked_sub(low).expect("ordered search interval").checked_div(2).expect("nonzero divisor");
        let mid = low.checked_add(half).expect("table index");
        let &(key, value) = fold::FOLD.get(mid).expect("search interval within table");
        match key.cmp(&char) {
            core::cmp::Ordering::Equal => return value,
            core::cmp::Ordering::Less => low = mid.checked_add(1).expect("table index"),
            core::cmp::Ordering::Greater => high = mid,
        }
    }
    char
}
