//! Checked store admission bounds (protocol/store.md, 5.4 and 6).
use crate::{accounts_v1, codec, component_v2, core_v6, notes_v1, people_v10, tasks_v9};
use jig_domain_core as core;

/// The application's largest encodings, before the component's key tag.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Application {
    pub version: u32,
    pub key: u32,
    pub value: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sizes {
    pub key: u32,
    pub value: u32,
    pub ops: u32,
    pub commit: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SizeRefusal {
    Key { needs: u32, has: u32 },
    Value { needs: u32, has: u32 },
    Ops { needs: u32, has: u32 },
    Commit { needs: u32, has: u32 },
    Page { needs: u32, has: u32 },
    Queue { needs: u32, has: u32 },
    QueuedBytes { needs: u64, has: u64 },
    Ceiling,
}

pub fn check(
    domain: &core::Limits,
    application: &Application,
    journal: &skein_lib::JournalLimits,
    store: &skein_kv::Limits,
    orphan_erases: u32,
) -> Result<Sizes, SizeRefusal> {
    let sizes = largest(domain, application, journal, orphan_erases, store.budget).ok_or(SizeRefusal::Ceiling)?;
    if sizes.key > store.key {
        return Err(SizeRefusal::Key { needs: sizes.key, has: store.key });
    }
    if sizes.value > store.value {
        return Err(SizeRefusal::Value { needs: sizes.value, has: store.value });
    }
    if sizes.ops > store.ops {
        return Err(SizeRefusal::Ops { needs: sizes.ops, has: store.ops });
    }
    if sizes.commit > store.commit {
        return Err(SizeRefusal::Commit { needs: sizes.commit, has: store.commit });
    }
    // skein-kv pages count the row's key and value payloads.
    let page = sizes.key.checked_add(sizes.value).ok_or(SizeRefusal::Ceiling)?;
    if page > store.page.bytes {
        return Err(SizeRefusal::Page { needs: page, has: store.page.bytes });
    }
    if journal.commits > store.queued {
        return Err(SizeRefusal::Queue { needs: journal.commits, has: store.queued });
    }
    let queued = u64::from(journal.commits).checked_mul(u64::from(sizes.commit)).ok_or(SizeRefusal::Ceiling)?;
    if queued > store.queued_bytes {
        return Err(SizeRefusal::QueuedBytes { needs: queued, has: store.queued_bytes });
    }
    Ok(sizes)
}

fn largest(
    domain: &core::Limits,
    application: &Application,
    journal: &skein_lib::JournalLimits,
    orphan_erases: u32,
    budget: u64,
) -> Option<Sizes> {
    let limits = codec::limits(domain)?;
    // A resources line has 32 fixed bytes, three per empty segment, and at
    // most two per payload byte (escaped zero). Notes bounds both count and sum.
    // The largest fixed key is a tasks update: tags plus five widened numbers.
    let key = domain.notes.pattern_bytes.checked_mul(5)?.checked_add(32)?.max(44).max(application.key.checked_add(1)?);
    // Tasks price additions and kept lines at dependencies, circle holds at circle;
    // people prices the same refusal names through codec::limits (store.md, 5.1).
    // Transcript bodies are frames in files; this sizes map values only.
    let mut map_core = limits.core;
    map_core.turn_record_transcript = 0;
    let value = u32::try_from(
        core_v6::Row::worst_case_bytes(&map_core)?
            .max(tasks_v9::Row::worst_case_bytes(&limits.tasks)?)
            .max(people_v10::Row::worst_case_bytes(&limits.people)?)
            .max(notes_v1::Row::worst_case_bytes(&limits.notes)?)
            .max(component_v2::worst_case_bytes(&component_v2::CEILINGS)?)
            .max(accounts_v1::worst_case_bytes(&accounts_v1::CEILINGS)?)
            .max(u64::from(application.value)),
    )
    .ok()?;
    // Each journal write can produce two turn places; the first commit also
    // inserts the format row and the installation's identifier. Each put
    // costs a tag and two u32 lengths; a skein-kv frame costs 22 header bytes
    // and four checksum bytes.
    let ops = crate::commits::most_ops(journal.writes, orphan_erases)?;
    let commit = u32::try_from(
        budget.checked_add(u64::from(key).checked_add(9)?.checked_mul(u64::from(ops))?)?.checked_add(26)?,
    )
    .ok()?;
    Some(Sizes { key, value, ops, commit })
}
