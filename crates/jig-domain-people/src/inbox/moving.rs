//! A conversation move retires its prior owner projection atomically.
use super::{Domain, Limits, Queue, Request, Stored, forget_chat};

/// Retire the old projection before the new task header rebuilds its census.
pub(crate) fn move_chat(
    domain: &mut Domain,
    limits: &Limits,
    task: u64,
    project: u32,
    from: u64,
    to: u64,
    out: &mut Queue<Request>,
) {
    let chat = domain.chat_totals.get(&task).expect("a moved conversation has a live census");
    assert!(chat.party == from && chat.project == project, "authentic move retires its previous requester");
    forget_chat(domain, limits, task, out);
    assert!(domain.chats_seen.insert((to, task), 0).is_ok(), "one position replaces the old requester's");
    out.push(Request::Save { record: Stored::ReadChat { party: to, task, through: 0 } });
}
