#!/usr/bin/env python3
"""Emit canonical simple-fold classes from regex-syntax's all-pairs table."""
import re
import sys
from pathlib import Path

source = Path(sys.argv[1]).read_text(encoding="utf-8")
assert "Unicode version: 16.0.0." in source
assert "pub const CASE_FOLDING_SIMPLE:" in source

def scalar(literal):
    value = literal[1:-1]
    if value.startswith("\\u{"):
        return chr(int(value[3:-1], 16))
    assert len(value) == 1, literal
    return value

literal = r"'(?:\\u\{[0-9a-fA-F]+\}|[^'\\])'"
rows = re.findall(r"\((" + literal + r"), &\[([^\]]*)\]\)", source)
assert rows, "expected all-pairs rows"
folds = {}
for item, peers in rows:
    char = scalar(item)
    members = [char] + [scalar(peer) for peer in re.findall(literal, peers)]
    least = min(members)
    if char != least:
        folds[char] = least

print("// Generated from Unicode 16.0.0's simple case folding via regex-syntax 0.8.11's")
print("// CASE_FOLDING_SIMPLE. Regenerate from the workspace root:")
print("// python3 -I scripts/dev/fold-table.py ~/.cargo/registry/src/*/regex-syntax-0.8.11/src/unicode_tables/case_folding_simple.rs > crates/jig-domain-people/src/names/fold.rs")
print("pub(super) const FOLD: &[(char, char)] = &[")
for char, least in sorted(folds.items()):
    print(f"    ('\\u{{{ord(char):x}}}', '\\u{{{ord(least):x}}}'),")
print("];")
