//! Objects are held once with their source sets; jig 20.3 builds this working set. Jig 21 adds proposal, escalation, question, person task and result card transitions. client/domain/client.md, 5.
pub mod escalation;
pub mod person_task;
pub mod proposal;
pub mod question;
pub mod result;
