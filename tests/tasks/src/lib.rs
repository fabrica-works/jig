//! Tasks with scripted executors and an atomic durable parent at the retained
//! production boundary (domain/tasks.md, 11; domain/engine.md, 7).
pub mod accounting_referee;
pub mod coverage;
pub mod referee;
pub mod stories;
pub mod trace;
mod world;
pub use world::{
    Frozen, LIMITS, RETRY, Reply, World, authority, run_story, run_story_branches, run_story_facts, run_story_traced,
    task,
};
