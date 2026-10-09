//! One in-flight document and a dirty mark per watch; contract.md, 7.
use crate::{Engine, Invalid, Rows, Up, payload, reads::list};
use jig_client::contract::{
    Actor, Call, Conversations, Delivery, Goals, Inbox, ObjectKey, Phase, Read, ReadValue, RunText, Subject, TaskLine,
    Tasks, Text, TurnName, WatchEnd, WatchEvent,
};
use skein_lib::{Time, Token, Wall};

#[derive(Debug)]
pub(crate) struct Watch {
    sign_in: u64,
    party: u64,
    subject: Subject,
    pub(crate) dirty: bool,
    in_flight: bool,
    heard: Time,
}
impl Engine {
    /// Admits one subject, then emits whole current values until one terminal.
    pub fn open(&mut self, now: Time, sign_in: u64, token: Token, subject: Subject) -> Result<(), Invalid> {
        if !payload::fits(&subject, &self.bounds) {
            return Err(Invalid::Payload);
        }
        if !self.watch_tokens.insert(token) {
            return Err(Invalid::Identity);
        }
        let Some(party) = self.truth.sign_ins.get(&sign_in).copied() else {
            self.queue(now, 0, Up::Ended { token, why: WatchEnd::SignedOut });
            return Ok(());
        };
        let party_count = self.watches.values().filter(|watch| watch.party == party).count();
        let why = if self.watches.len() >= self.bounds.watches as usize
            || party_count >= self.bounds.watches_per_party as usize
        {
            Some(WatchEnd::Busy)
        } else {
            self.subject_end(party, &subject, false)
        };
        if let Some(why) = why {
            self.queue(now, self.next_commit - 1, Up::Ended { token, why });
            return Ok(());
        }
        self.watches.insert(token, Watch { sign_in, party, subject, dirty: true, in_flight: false, heard: now });
        self.queue(now, 0, Up::Watch { token, event: WatchEvent::Opened });
        Ok(())
    }
    /// Closes a live watch once; queued deliveries are replaced by its terminal.
    pub fn close(&mut self, now: Time, token: Token) {
        self.end_watch(now, token, WatchEnd::Closed, 0);
    }
    /// The root took the previous delivery; a dirty current value may now be made.
    pub fn taken(&mut self, token: Token) {
        if let Some(watch) = self.watches.get_mut(&token) {
            watch.in_flight = false;
        }
    }
    /// Advances the same run watch with the client's turn name and absolute offset.
    pub fn continue_run(&mut self, token: Token, turn: Option<TurnName>, offset: u32) -> Result<(), Invalid> {
        let watch = self.watches.get_mut(&token).ok_or(Invalid::Identity)?;
        let Subject::Run { turn: current, offset: through, .. } = &mut watch.subject else {
            return Err(Invalid::Identity);
        };
        *current = turn;
        *through = offset;
        watch.in_flight = false;
        watch.dirty = true;
        Ok(())
    }
    pub(crate) fn end_watch(&mut self, now: Time, token: Token, why: WatchEnd, commit: u64) {
        if self.watches.remove(&token).is_some() {
            self.due.retain(|due| !matches!(due.up, Up::Watch { token: id, .. } if id == token));
            self.queue(now, commit, Up::Ended { token, why });
        }
    }
    pub(crate) fn prepare_watches(&mut self, now: Time) {
        // A delivery is made only at its durability barrier. No stale snapshot is
        // queued while a newer commit can supersede it.
        let durable = self.commits.last().map_or(0, |commit| commit.number);
        let tokens: Vec<_> = self.watches.keys().copied().collect();
        for token in tokens {
            let mut watch = self.watches.remove(&token).expect("selected watch exists");
            let end = if self.truth.sign_ins.get(&watch.sign_in) == Some(&watch.party) {
                self.subject_end(watch.party, &watch.subject, true)
            } else {
                Some(WatchEnd::SignedOut)
            };
            if let Some(why) = end {
                self.watches.insert(token, watch);
                self.end_watch(now, token, why, self.next_commit - 1);
                continue;
            }
            let provisional = matches!(watch.subject, Subject::Run { .. });
            if !watch.in_flight && watch.dirty && (provisional || durable == self.next_commit - 1) {
                match self.delivery(watch.party, &watch.subject) {
                    Ok(Some(value)) => {
                        watch.in_flight = true;
                        watch.dirty = false;
                        watch.heard = now;
                        self.queue(
                            now,
                            if provisional { 0 } else { durable },
                            Up::Watch { token, event: WatchEvent::Delivered(value) },
                        );
                    }
                    Ok(None) => {
                        watch.dirty = false;
                    }
                    Err(why) => {
                        self.watches.insert(token, watch);
                        self.end_watch(now, token, why, self.next_commit - 1);
                        continue;
                    }
                }
            }
            if now >= watch.heard.saturating_add(self.bounds.heartbeat) {
                watch.heard = now;
                self.queue(now, 0, Up::Watch { token, event: WatchEvent::Alive });
            }
            self.watches.insert(token, watch);
        }
    }
    pub(crate) fn watch_due(&self) -> impl Iterator<Item = Time> + '_ {
        self.watches.values().map(|watch| {
            let heartbeat = watch.heard.saturating_add(self.bounds.heartbeat);
            if watch.dirty && !watch.in_flight {
                let barrier = if matches!(watch.subject, Subject::Run { .. }) {
                    watch.heard
                } else {
                    self.pending.last().map_or(watch.heard, |pending| pending.due)
                };
                heartbeat.min(barrier)
            } else {
                heartbeat
            }
        })
    }
    fn subject_end(&self, party: u64, subject: &Subject, admitted: bool) -> Option<WatchEnd> {
        let hidden = if admitted { WatchEnd::NotAllowed } else { WatchEnd::Unknown };
        let tasks: Vec<u64> = match subject {
            Subject::Task { task } | Subject::Tree { task, .. } | Subject::Run { task, .. } => vec![*task],
            Subject::Chips { tasks } => tasks.iter().copied().collect(),
            Subject::Goals { project } | Subject::Tasks { project, .. } => {
                return (!self.truth.visible(party, *project)).then_some(hidden);
            }
            Subject::Frame => return (!self.truth.frames.contains_key(&party)).then_some(hidden),
            Subject::Inbox { .. } | Subject::Conversations => return None,
            Subject::Deployment => return (!self.read_visible(party, &Read::Deployment)).then_some(hidden),
        };
        for task in tasks {
            let Some(ReadValue::Task(doc)) = self.truth.objects.get(&ObjectKey::Task(task)) else {
                return Some(if admitted { WatchEnd::SubjectEnded } else { WatchEnd::Unknown });
            };
            if !self.truth.visible(party, doc.project) {
                return Some(hidden);
            }
            if ended(doc.chip.phase) {
                return Some(WatchEnd::SubjectEnded);
            }
        }
        None
    }
    #[expect(clippy::too_many_lines, reason = "each contract subject has a distinct whole document")]
    fn delivery(&mut self, party: u64, subject: &Subject) -> Result<Option<Delivery>, WatchEnd> {
        let mut value = match subject {
            Subject::Frame => Delivery::Frame(Box::new(self.truth.frames[&party].clone())),
            Subject::Task { task } => {
                let Some(ReadValue::Task(doc)) = self.truth.objects.get(&ObjectKey::Task(*task)) else {
                    return Err(WatchEnd::Unknown);
                };
                Delivery::Task(doc.clone())
            }
            Subject::Tree { task, window } => {
                let mut tree = self.truth.trees.get(task).cloned().ok_or(WatchEnd::Unknown)?;
                let nodes: Vec<_> = tree.nodes.iter().cloned().collect();
                let count = nodes.len().min((*window).min(self.bounds.rows) as usize);
                for node in &nodes[count..] {
                    fold(&mut tree.folded, node.chip.phase);
                }
                tree.nodes = list(&nodes[..count]);
                Delivery::Tree(Box::new(tree))
            }
            Subject::Chips { tasks } => {
                let mut chips = Vec::new();
                for task in tasks {
                    let Some(ReadValue::Task(doc)) = self.truth.objects.get(&ObjectKey::Task(*task)) else {
                        return Err(WatchEnd::Unknown);
                    };
                    chips.push(doc.chip.clone());
                }
                Delivery::Chips(list(&chips))
            }
            Subject::Inbox { window } => {
                let read = Read::Inbox { project: None, kind: None, goal: None, after: None };
                let Rows::Inbox(entries) = self.rows_for(party, &read) else {
                    unreachable!("inbox rows");
                };
                let count = entries.len().min((*window).min(self.bounds.rows) as usize);
                let frame = &self.truth.frames[&party];
                Delivery::Inbox(Box::new(Inbox {
                    entries: list(&entries[..count]),
                    needs: frame.needs,
                    updates: frame.updates,
                    results_seen: self.truth.results_seen.get(&party).copied().unwrap_or(0),
                    replies_seen: list(self.truth.replies_seen.get(&party).map_or(&[], Vec::as_slice)),
                    more: count < entries.len(),
                }))
            }
            Subject::Goals { project } => {
                let Rows::Goals(rows) =
                    self.rows_for(party, &Read::Goals { project: *project, ended: false, after: None })
                else {
                    unreachable!("goal rows");
                };
                let count = rows.len().min(self.bounds.rows as usize);
                Delivery::Goals(Box::new(Goals {
                    project: *project,
                    rows: list(&rows[..count]),
                    more: count < rows.len(),
                }))
            }
            Subject::Conversations => {
                let Rows::Conversations(rows) =
                    self.rows_for(party, &Read::Conversations { project: None, after: None })
                else {
                    unreachable!("conversation rows");
                };
                let live: Vec<_> = rows.into_iter().filter(|line| !ended(line.chip.phase)).collect();
                let count = live.len().min(self.bounds.rows as usize);
                Delivery::Conversations(Box::new(Conversations {
                    rows: list(&live[..count]),
                    more: count < live.len(),
                }))
            }
            Subject::Tasks { project, filter, window } => {
                let mut rows = Vec::new();
                for doc in self.truth.objects.values() {
                    if let ReadValue::Task(task) = doc {
                        let asked_by_me = task.requester == Actor::Party(party);
                        let goal = task
                            .lineage
                            .iter()
                            .find(|id| {
                                self.truth
                                    .objects
                                    .get(&ObjectKey::Task(**id))
                                    .is_some_and(|v| matches!(v, ReadValue::Task(doc) if doc.goal))
                            })
                            .copied();
                        if task.project != *project
                            || ended(task.chip.phase)
                            || filter.phase.is_some_and(|phase| phase != task.chip.phase)
                            || filter.executor.is_some_and(|executor| executor != task.chip.executor)
                            || filter.goal.is_some_and(|wanted| goal != Some(wanted))
                            || (filter.asked_by_me && !asked_by_me)
                        {
                            continue;
                        }
                        rows.push(TaskLine {
                            asked_by_me,
                            chip: task.chip.clone(),
                            goal,
                            requester: task.requester,
                            changed: task.changed,
                        });
                    }
                }
                rows.sort_by(|a, b| b.changed.cmp(&a.changed).then(a.chip.task.cmp(&b.chip.task)));
                let count = rows.len().min((*window).min(self.bounds.rows) as usize);
                Delivery::Tasks(Box::new(Tasks {
                    project: *project,
                    filter: *filter,
                    rows: list(&rows[..count]),
                    more: count < rows.len(),
                }))
            }
            Subject::Run { task, turn, offset } => {
                let Some(run) = self.runs.get(task) else {
                    return Ok(None);
                };
                let from = if *turn == Some(run.turn) { (*offset).max(run.from).min(run.through) } else { run.from };
                let start = (from - run.from) as usize;
                let mut value = run.clone();
                value.from = from;
                value.text = Text::new(run.text.bytes()[start..].to_vec().into_boxed_slice(), self.bounds.text)
                    .expect("retained run fits");
                Delivery::Run(Box::new(value))
            }
            Subject::Deployment => {
                Delivery::Deployment(Box::new(self.truth.deployment.clone().ok_or(WatchEnd::Unknown)?))
            }
        };
        loop {
            if payload::fits(&value, &self.bounds) {
                return Ok(Some(value));
            }
            match &mut value {
                Delivery::Inbox(doc) => {
                    if !shorten(&mut doc.entries) {
                        return Err(WatchEnd::Busy);
                    }
                    doc.more = true;
                }
                Delivery::Goals(doc) => {
                    if !shorten(&mut doc.rows) {
                        return Err(WatchEnd::Busy);
                    }
                    doc.more = true;
                }
                Delivery::Tasks(doc) => {
                    if !shorten(&mut doc.rows) {
                        return Err(WatchEnd::Busy);
                    }
                    doc.more = true;
                }
                Delivery::Conversations(doc) => {
                    if !shorten(&mut doc.rows) {
                        return Err(WatchEnd::Busy);
                    }
                    doc.more = true;
                }
                Delivery::Tree(doc) => {
                    let nodes: Vec<_> = doc.nodes.iter().cloned().collect();
                    let Some(last) = nodes.last() else {
                        return Err(WatchEnd::Busy);
                    };
                    fold(&mut doc.folded, last.chip.phase);
                    doc.nodes = list(&nodes[..nodes.len() - 1]);
                }
                Delivery::Frame(_)
                | Delivery::Task(_)
                | Delivery::Chips(_)
                | Delivery::Run(_)
                | Delivery::Deployment(_) => return Err(WatchEnd::Busy),
            }
        }
    }
    /// Appends provisional bytes to the current writing turn, retaining its bounded tail.
    /// A new name starts at zero; activity is the only carrier of committed turns.
    pub fn write(
        &mut self,
        now: Time,
        task: u64,
        turn: TurnName,
        began: Wall,
        bytes: &Text,
        call: Option<Call>,
    ) -> Result<(), Invalid> {
        if !payload::fits(bytes, &self.bounds) || !payload::fits(&call, &self.bounds) {
            return Err(Invalid::Payload);
        }
        if !self.truth.objects.contains_key(&ObjectKey::Task(task)) {
            return Err(Invalid::Identity);
        }
        let old = self.runs.get(&task).filter(|run| run.turn == turn);
        let mut text = old.map_or_else(Vec::new, |run| run.text.bytes().to_vec());
        let through = old
            .map_or(0, |run| run.through)
            .checked_add(u32::try_from(bytes.bytes().len()).map_err(|_| Invalid::Payload)?)
            .ok_or(Invalid::Payload)?;
        text.extend_from_slice(bytes.bytes());
        let kept = text.len().min(self.bounds.text as usize);
        let from = through - u32::try_from(kept).expect("text bound fits");
        let run = RunText {
            attempt: turn.attempt,
            began,
            turn,
            from,
            through,
            text: Text::new(text[text.len() - kept..].to_vec().into_boxed_slice(), self.bounds.text)
                .expect("bounded tail"),
            cut: from != 0,
            call,
        };
        if !payload::fits(&run, &self.bounds) {
            return Err(Invalid::Payload);
        }
        self.runs.insert(task, run);
        for watch in self.watches.values_mut() {
            if matches!(watch.subject, Subject::Run { task:id, .. } if id == task) {
                watch.dirty = true;
                watch.heard = now;
            }
        }
        Ok(())
    }
}
fn ended(phase: Phase) -> bool {
    matches!(phase, Phase::Done | Phase::Failed | Phase::Cancelled)
}

fn shorten<T: Clone>(rows: &mut skein_lib::List<T>) -> bool {
    let copy: Vec<_> = rows.iter().cloned().collect();
    if copy.is_empty() {
        return false;
    }
    *rows = list(&copy[..copy.len() - 1]);
    true
}
fn fold(counts: &mut jig_client::contract::PhaseCounts, phase: Phase) {
    let count = match phase {
        Phase::Waiting | Phase::Idle | Phase::Due | Phase::Preparing | Phase::Claimed | Phase::BackingOff => {
            &mut counts.waiting
        }
        Phase::Running | Phase::Active => &mut counts.running,
        Phase::Held => &mut counts.held,
        Phase::Closing => &mut counts.closing,
        Phase::Done => &mut counts.done,
        Phase::Failed => &mut counts.failed,
        Phase::Cancelled => &mut counts.cancelled,
    };
    *count = count.saturating_add(1);
}
