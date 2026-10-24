//! Refusals pinned to their design text (store.md 2 and 10.1).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "text assertions use ordinary Rust (programming-model.md 10.2)"
)]
use crate::{Generations, Older, OpenRefusal};

#[expect(clippy::disallowed_macros, reason = "the task pins refusal words to the design itself")]
const DESIGN: &str = include_str!("../../../../docs/design/protocol/store.md");
fn unwrapped(start: &str, end: &str) -> String {
    let text = DESIGN.split_once(start).expect("design's refusal start").1;
    let text = text.split_once(end).expect("design's refusal end").0;
    text.lines().map(|line| line.trim().trim_start_matches("> ")).collect::<Vec<_>>().join(" ")
}
fn says(refusal: OpenRefusal, start: &str, end: &str) {
    assert_eq!(refusal.words().join("`<deployment>`"), unwrapped(start, end));
}
#[test]
fn todays_layouts_refusal_says_to_move_its_three_directories() {
    says(OpenRefusal::OlderBuild { why: Older::Directory }, "  > ", "\n\n- **Each `Older`");
}
#[test]
fn a_busy_refusal_says_to_stop_the_other_process() {
    says(OpenRefusal::Generations { why: Generations::Busy }, "- `Busy`: \"", "\"");
}
#[test]
fn an_unreadable_refusal_says_to_restore_or_name_a_generation() {
    says(OpenRefusal::Generations { why: Generations::Unreadable }, "- `Unreadable`: \"", "\"");
}
#[test]
fn a_missing_refusal_says_to_restore_or_name_a_generation() {
    says(OpenRefusal::Generations { why: Generations::Missing { generation: 7 } }, "- `Missing`: \"", "\"");
}
#[test]
fn an_unnamed_refusal_says_to_name_its_generation() {
    says(OpenRefusal::Generations { why: Generations::Unnamed }, "- `Unnamed`: \"", "\"");
}
