#![expect(clippy::manual_map, reason = "step code matches optional owner facts without closures")]
//! Whole live document facts (protocol/client.md, 3 and 7).
use super::{chips, pieces, task, w};
use alloc::boxed::Box;
use jig_domain_core::documents as d;
use jig_domain_views as views;
use skein_lib::{List, Writer, bytes};
pub fn current_call(value: &views::Call, limits: &w::Limits) -> Result<w::CurrentCall, w::Problem> {
    w::CurrentCall::new(
        limits,
        w::CurrentCallParts {
            identity: w::CallIdentity::new(
                limits,
                w::CallIdentityParts {
                    owner: value.identity.owner.raw(),
                    sequence: value.identity.sequence,
                    retry: value.identity.retry,
                    id: bytes::copy_of(&value.identity.id),
                },
            )?,
            tool: bytes::copy_of(&value.tool),
            input: bytes::copy_of(&value.input),
            cut: value.cut,
            began: match value.began {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
        },
    )
}
pub fn writing(value: &views::Writing, limits: &w::Limits) -> Result<w::RunText, w::Problem> {
    w::RunText::new(
        limits,
        w::RunTextParts {
            attempt: value.turn.attempt,
            began: None,
            turn: w::ProvisionalTurnName::new(
                limits,
                w::ProvisionalTurnNameParts {
                    attempt: value.turn.attempt,
                    number: value.turn.turn,
                    retry: value.turn.retry,
                },
            )?,
            from: value.from,
            through: value.to,
            text: bytes::copy_of(&value.text),
            cut: value.cut,
            call: match &value.call {
                Some(call) => Some(w::LiveCall::Raw(w::LiveCallRaw::new(
                    limits,
                    w::LiveCallRawParts { value: current_call(call, limits)? },
                )?)),
                None => None,
            },
        },
    )
}
pub fn run(value: &d::Run, limits: &w::Limits) -> Result<w::RunState, w::Problem> {
    w::RunState::new(
        limits,
        w::RunStateParts {
            task: value.task,
            attempt: value.attempt,
            committed_turn: value.committed_turn,
            writing: match &value.writing {
                Some(item) => Some(writing(item, limits)?),
                None => None,
            },
            call: match &value.call {
                Some(item) => Some(current_call(item, limits)?),
                None => None,
            },
        },
    )
}
pub fn count(value: d::Count, limits: &w::Limits) -> Result<w::Count, w::Problem> {
    let (number, limited) = match value {
        d::Count::Known(n) => (n, false),
        d::Count::AtLeast(n) => (n, true),
    };
    w::Count::new(limits, w::CountParts { value: number, limited })
}
pub fn phases(value: d::PhaseCounts, limits: &w::Limits) -> Result<w::OwnerPhaseCounts, w::Problem> {
    w::OwnerPhaseCounts::new(
        limits,
        w::OwnerPhaseCountsParts {
            waiting: value.waiting,
            idle: value.idle,
            starting: value.starting,
            running: value.running,
            backing_off: value.backing_off,
            closing: value.closing,
            held: value.held,
            done: value.done,
            failed: value.failed,
            cancelled: value.cancelled,
        },
    )
}

pub fn tree_row(value: &d::TreeRow, limits: &w::Limits) -> Result<w::OwnerTreeRow, w::Problem> {
    let mut proposals = List::with_capacity(value.proposals.len());
    for row in &value.proposals {
        proposals.push(task::proposal(row, limits)?).expect("supplied proposals fit");
    }
    let mut dependencies = List::with_capacity(value.dependencies.len());
    for row in &value.dependencies {
        dependencies.push(task::link(row, limits)?).expect("supplied dependencies fit");
    }
    let dependencies =
        task::links_pieces(&dependencies, limits.owner_tree_row_dependencies, w::Path::OwnerTreeRowDependencies)?;
    w::OwnerTreeRow::new(
        limits,
        w::OwnerTreeRowParts {
            chip: chips::chip(&value.chip, limits)?,
            parent: value.parent,
            depth: value.depth,
            proposals,
            spent: value.spent,
            dependencies,
            cut: value.cut,
        },
    )
}
pub fn tree(value: &d::Tree, next: Option<w::Cursor>, limits: &w::Limits) -> Result<w::OwnerTree, w::Problem> {
    let mut rows = List::with_capacity(value.rows.len());
    for row in &value.rows {
        rows.push(tree_row(row, limits)?).expect("supplied rows fit");
    }
    let rows = tree_pieces(&rows, limits.owner_tree_rows, w::Path::OwnerTreeRows)?;
    let mut folded = List::with_capacity(value.folded.len());
    for row in &value.folded {
        folded
            .push(w::OwnerTreeFold::new(
                limits,
                w::OwnerTreeFoldParts {
                    parent: row.parent,
                    phases: phases(row.phases, limits)?,
                    held_below: row.held_below,
                },
            )?)
            .expect("supplied folds fit");
    }
    w::OwnerTree::new(
        limits,
        w::OwnerTreeParts { root: chips::chip(&value.root, limits)?, rows, folded, held_below: value.held_below, next },
    )
}

fn tree_pieces(rows: &List<w::OwnerTreeRow>, bound: u32, path: w::Path) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    pieces(&encoded, bound, path)
}
