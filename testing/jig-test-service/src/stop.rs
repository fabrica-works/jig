//! A failed commit ends the engine before any later output leaves (service.md, 5).
use crate::{End, Engine, Stop};
use jig_protocol_config::{Path, Problems, What};
use jig_test_domain as root;

pub(super) fn failed(engine: &mut Engine, number: u64) {
    engine.step(root::Event::Failed { number });
    engine.end = Some(End::Stopped(Stop::CommitFailed { number }));
}
pub(super) fn invalid_restore(engine: &mut Engine) {
    unavailable(engine, b"deployment");
}
pub(super) fn unavailable(engine: &mut Engine, member: &[u8]) {
    let mut problems = Problems::empty();
    problems.push(Path::root().member(member), What::Refers);
    engine.end = Some(End::Stopped(Stop::Start(problems)));
}
