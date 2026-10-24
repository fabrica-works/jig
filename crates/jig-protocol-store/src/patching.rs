//! Patching at open (protocol/store.md 10.2 and 10.3): the owners' patch
//! sets, the plan across them, one machine per kept family, a record's
//! walk, and the phase that runs them over a copy.
#[cfg_attr(not(test), expect(dead_code, reason = "task B wires the patch phase into open"))]
pub(crate) mod machines;
pub(crate) mod phase;
#[cfg_attr(not(test), expect(dead_code, reason = "task B wires the patch phase into open"))]
pub(crate) mod plan;
pub mod sets;
#[cfg_attr(not(test), expect(dead_code, reason = "task B wires the patch phase into open"))]
pub(crate) mod walk;
