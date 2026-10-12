//! Text answers from the core's parts (protocol/channel.md, section 6.3).
use crate::{Problem, Rendered, Why};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_core::{self as core, CallPart};
use skein_lib::{Decimal, List, Writer};

fn text(parts: &[&[u8]], error: bool) -> Rendered {
    let mut size = 0_usize;
    for part in parts {
        size = size.checked_add(part.len()).expect("bounded text parts");
    }
    let mut writer = Writer::new(size);
    for part in parts {
        writer.put(part).expect("measured text parts");
    }
    Rendered { error, text: writer.finish() }
}
#[must_use]
pub fn problem(tool: &[u8], problem: &Problem) -> Rendered {
    let why: &[u8] = match problem.why {
        Why::Malformed => b"is malformed",
        Why::Missing => b"is missing",
        Why::Type => b"has the wrong type",
        Why::Range => b"is outside its range",
        Why::TooLarge => b"is too large",
    };
    if problem.field.is_empty() {
        text(&[tool, b": the input ", why], true)
    } else {
        text(&[tool, b": the field ", problem.field, b" ", why], true)
    }
}
#[must_use]
pub fn render(part: &CallPart) -> Rendered {
    match part {
        CallPart::Effect { entry, outcome, .. } => {
            let status: &[u8] = match outcome {
                None => b"pending",
                Some(core::connector::OutboxOutcome::Made) => b"made",
                Some(core::connector::OutboxOutcome::Failed) => b"failed",
                Some(core::connector::OutboxOutcome::Withdrawn) => b"withdrawn",
                Some(core::connector::OutboxOutcome::NotMade) => b"not made",
                Some(core::connector::OutboxOutcome::Uncertain) => b"uncertain",
                Some(core::connector::OutboxOutcome::Held { .. }) => b"held",
            };
            text(&[b"Effect ", Decimal::of(*entry).as_bytes(), b": ", status], false)
        }
        CallPart::Connector { .. } => text(&[b"Connector answer"], false),
        CallPart::Proposed { proposal } => text(&[b"Proposed ", Decimal::of(*proposal).as_bytes()], false),
        CallPart::ProposalDecided { proposal, .. } => {
            text(&[b"Decided proposal ", Decimal::of(*proposal).as_bytes()], false)
        }
        CallPart::Sent { message } => text(&[b"Sent message ", Decimal::of(*message).as_bytes()], false),
        CallPart::Subscribed { subscription } => text(&[b"Subscribed ", Decimal::of(*subscription).as_bytes()], false),
        CallPart::Delegated(numbers) => delegated(numbers),
        CallPart::NoteWritten { name, revision } => text(
            &[
                b"Wrote note ",
                Decimal::of(*name).as_bytes(),
                b" revision ",
                Decimal::of(u64::from(*revision)).as_bytes(),
            ],
            false,
        ),
        CallPart::Controlled | CallPart::Introduced | CallPart::Unsubscribed | CallPart::EscalationDecided { .. } => {
            text(&[b"Done"], false)
        }
        CallPart::EffectDenied { answer, findings }
        | CallPart::ToolDenied { answer, findings }
        | CallPart::DelegationDenied { answer, findings } => denied(*answer, findings),
        CallPart::ControlDenied { answer } => denied(*answer, &[]),
        CallPart::EscalationRefused(problem)
        | CallPart::ProposalRefused(problem)
        | CallPart::ControlRefused(problem)
        | CallPart::MessageRefused(problem)
        | CallPart::SubscriptionRefused(problem)
        | CallPart::DelegationRefused(problem) => task_problem(problem),
        CallPart::NoteRecalled { entries, more } => Rendered { error: false, text: notes(entries, *more) },
        CallPart::NoteRefused(why) => {
            let why: &[u8] = match why {
                jig_domain_notes::Refusal::Busy => b"busy",
                jig_domain_notes::Refusal::Oversized => b"too large",
                jig_domain_notes::Refusal::Full => b"scope full",
                jig_domain_notes::Refusal::Exists => b"name exists",
                jig_domain_notes::Refusal::Missing => b"missing",
                jig_domain_notes::Refusal::Moved { .. } => b"revision changed",
                jig_domain_notes::Refusal::RevisionExhausted => b"revision exhausted",
            };
            text(&[b"Note refused: ", why], true)
        }
        CallPart::Unavailable => text(&[b"Unavailable"], true),
    }
}
fn delegated(numbers: &[u64]) -> Rendered {
    let mut size = b"Delegated tasks:".len();
    for number in numbers {
        size =
            size.checked_add(1).expect("separator").checked_add(Decimal::of(*number).as_bytes().len()).expect("number");
    }
    let mut writer = Writer::new(size);
    writer.put(b"Delegated tasks:").expect("heading");
    for number in numbers {
        writer.put(b" ").expect("separator");
        writer.put(Decimal::of(*number).as_bytes()).expect("number");
    }
    Rendered { error: false, text: writer.finish() }
}
fn denied(answer: authority::Answer, findings: &[authority::Finding]) -> Rendered {
    let status: &[u8] = match answer {
        authority::Answer::Allow => b"Allowed",
        authority::Answer::Wait => b"Waiting",
        authority::Answer::Propose => b"Beyond authority",
        authority::Answer::Refuse => b"Refused",
    };
    let count = u32::try_from(findings.len())
        .expect("bounded findings")
        .checked_mul(8)
        .expect("eight authority components")
        .checked_add(1)
        .expect("status");
    let mut parts = List::with_capacity(count);
    parts.push(Box::from(status)).expect("status slot");
    for finding in findings {
        match finding {
            authority::Finding::Authority { lacks, .. } => {
                for (missing, name) in [
                    (lacks.tools, b"tools".as_slice()),
                    (lacks.grants, b"grants"),
                    (lacks.executors, b"executors"),
                    (lacks.tasks, b"tasks"),
                    (lacks.depth, b"depth"),
                    (lacks.spend, b"spend"),
                    (lacks.deadline, b"deadline"),
                    (lacks.notes, b"notes"),
                ] {
                    if missing {
                        parts.push(text(&[b"; missing ", name], true).text).expect("component slot");
                    }
                }
            }
            other @ (authority::Finding::Oversized
            | authority::Finding::UnknownProject
            | authority::Finding::UnknownRole
            | authority::Finding::Executor { .. }
            | authority::Finding::Tasks { .. }
            | authority::Finding::Spend { .. }
            | authority::Finding::Arithmetic
            | authority::Finding::RunBudget
            | authority::Finding::RunCap
            | authority::Finding::Deadline
            | authority::Finding::Account
            | authority::Finding::Writer
            | authority::Finding::Tool
            | authority::Finding::Grant { .. }
            | authority::Finding::Price { .. }
            | authority::Finding::ResourceAccess
            | authority::Finding::ResourceUnavailable
            | authority::Finding::Reference
            | authority::Finding::Scope { .. }
            | authority::Finding::Required { .. }
            | authority::Finding::Failed { .. }
            | authority::Finding::Unguarded { .. }
            | authority::Finding::Unpermitted
            | authority::Finding::Undecidable
            | authority::Finding::PeriodSpend) => {
                parts.push(text(&[b"; ", finding_text(other)], true).text).expect("finding slot");
            }
        }
    }
    let mut bytes = 0_usize;
    for part in &parts {
        bytes = bytes.checked_add(part.len()).expect("bounded findings text");
    }
    let mut writer = Writer::new(bytes);
    for part in &parts {
        writer.put(part).expect("measured findings");
    }
    Rendered { error: true, text: writer.finish() }
}
fn finding_text(finding: &authority::Finding) -> &'static [u8] {
    match finding {
        authority::Finding::Oversized => b"input too large",
        authority::Finding::UnknownProject => b"unknown project",
        authority::Finding::UnknownRole => b"unknown role",
        authority::Finding::Authority { .. } => b"missing authority",
        authority::Finding::Executor { .. } => b"executor not permitted",
        authority::Finding::Tasks { .. } => b"task capacity",
        authority::Finding::Spend { .. } => b"spend capacity",
        authority::Finding::Arithmetic => b"unrepresentable accounting",
        authority::Finding::RunBudget => b"run budget too small",
        authority::Finding::RunCap => b"run budget exceeds cap",
        authority::Finding::Deadline => b"deadline passed",
        authority::Finding::Account => b"account unavailable",
        authority::Finding::Writer => b"writer unavailable",
        authority::Finding::Tool => b"missing tool authority",
        authority::Finding::Grant { .. } => b"missing grant",
        authority::Finding::Price { .. } => b"price exceeds funding",
        authority::Finding::ResourceAccess => b"resource access denied",
        authority::Finding::ResourceUnavailable => b"resource unavailable",
        authority::Finding::Reference => b"missing reference",
        authority::Finding::Scope { .. } => b"missing note scope",
        authority::Finding::Required { .. } => b"required fact missing",
        authority::Finding::Failed { .. } => b"required fact failed",
        authority::Finding::Unguarded { .. } => b"required guard missing",
        authority::Finding::Unpermitted => b"permission missing",
        authority::Finding::Undecidable => b"decision permission missing",
        authority::Finding::PeriodSpend => b"period spend exceeds ceiling",
    }
}

fn task_problem(problem: &jig_domain_tasks::Problem) -> Rendered {
    let why: &[u8] = match problem.why {
        jig_domain_tasks::Refusal::Unproposable => b"no proposal could be accepted",
        jig_domain_tasks::Refusal::PersonRevision { .. } => b"person revision",
        jig_domain_tasks::Refusal::ResourceUnavailable => b"resource unavailable",
        jig_domain_tasks::Refusal::NotReady => b"not ready",
        jig_domain_tasks::Refusal::Unknown => b"unknown",
        jig_domain_tasks::Refusal::Busy => b"busy",
        jig_domain_tasks::Refusal::Duplicate => b"duplicate",
        jig_domain_tasks::Refusal::Empty => b"empty",
        jig_domain_tasks::Refusal::Batch => b"batch",
        jig_domain_tasks::Refusal::Live => b"live",
        jig_domain_tasks::Refusal::Project => b"project",
        jig_domain_tasks::Refusal::Tree => b"tree",
        jig_domain_tasks::Refusal::Depth => b"depth",
        jig_domain_tasks::Refusal::Delegates => b"delegates",
        jig_domain_tasks::Refusal::Reference => b"reference",
        jig_domain_tasks::Refusal::Subscription => b"subscription",
        jig_domain_tasks::Refusal::Dependencies => b"dependencies",
        jig_domain_tasks::Refusal::Cycle => b"cycle",
        jig_domain_tasks::Refusal::Executor => b"executor",
        jig_domain_tasks::Refusal::Spec => b"spec",
        jig_domain_tasks::Refusal::Contract => b"contract",
        jig_domain_tasks::Refusal::AuthorityShape => b"authority shape",
        jig_domain_tasks::Refusal::Inputs => b"inputs",
        jig_domain_tasks::Refusal::State => b"state",
        jig_domain_tasks::Refusal::Attempt => b"attempt",
        jig_domain_tasks::Refusal::LiveDelegates => b"live delegates",
        jig_domain_tasks::Refusal::Restore => b"restore",
        jig_domain_tasks::Refusal::Read => b"read",
        jig_domain_tasks::Refusal::Turn => b"turn",
        jig_domain_tasks::Refusal::Funding => b"funding",
        jig_domain_tasks::Refusal::HoldKind => b"hold kind",
        jig_domain_tasks::Refusal::HoldTaken => b"hold taken",
        jig_domain_tasks::Refusal::Holds => b"holds",
    };
    text(&[b"Refused: ", why], true)
}

fn notes(entries: &[jig_domain_notes::Entry], more: bool) -> Box<[u8]> {
    let tail = if more { b"\nMore notes remain.".as_slice() } else { b"\nEnd of notes.".as_slice() };
    let mut bytes = b"Notes".len().checked_add(tail.len()).expect("page framing");
    for entry in entries {
        bytes = bytes
            .checked_add(b"\nNote ".len())
            .expect("note heading")
            .checked_add(Decimal::of(entry.name).as_bytes().len())
            .expect("note identity")
            .checked_add(b" revision ".len())
            .expect("revision heading")
            .checked_add(Decimal::of(u64::from(entry.revision)).as_bytes().len())
            .expect("revision")
            .checked_add(2)
            .expect("line breaks")
            .checked_add(entry.description.len())
            .expect("description")
            .checked_add(entry.body.len())
            .expect("body")
            .checked_add(b"\nReferences:".len())
            .expect("references heading");
        for reference in &entry.references {
            bytes = bytes
                .checked_add(1)
                .expect("reference separator")
                .checked_add(Decimal::of(*reference).as_bytes().len())
                .expect("reference");
        }
    }
    let mut writer = Writer::new(bytes);
    writer.put(b"Notes").expect("page heading");
    for entry in entries {
        writer.put(b"\nNote ").expect("note heading");
        writer.put(Decimal::of(entry.name).as_bytes()).expect("note identity");
        writer.put(b" revision ").expect("revision heading");
        writer.put(Decimal::of(u64::from(entry.revision)).as_bytes()).expect("revision");
        writer.put(b"\n").expect("line break");
        writer.put(&entry.description).expect("description");
        writer.put(b"\n").expect("line break");
        writer.put(&entry.body).expect("body");
        writer.put(b"\nReferences:").expect("references heading");
        for reference in &entry.references {
            writer.put(b" ").expect("reference separator");
            writer.put(Decimal::of(*reference).as_bytes()).expect("reference");
        }
    }
    writer.put(tail).expect("page tail");
    writer.finish()
}
