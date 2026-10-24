//! Pure patch planning and stopped record walks (protocol/store.md 10.3).
#[cfg_attr(not(test), expect(dead_code, reason = "task B wires the patch phase into open"))]
pub(crate) mod machines;
#[cfg_attr(not(test), expect(dead_code, reason = "task B wires the patch phase into open"))]
pub(crate) mod plan;
pub mod sets;
#[cfg_attr(not(test), expect(dead_code, reason = "task B wires the patch phase into open"))]
pub(crate) mod walk;
