//! Reads bounded by durable places, walking complete frames (store.md, 8.3).
use super::{File, Stage, frame};
use crate::{Arrived, Below, Event, Limits, Phase, Store, TranscriptRead, codec, keys, open};
use alloc::boxed::Box;
use jig_domain_core::TurnRecord;
use skein_io::file;
use skein_lib::{List, Map, Queue, Reader, Token, Writer};
pub(crate) const OWNER: Token = Token::new(0x5453);
#[derive(Debug)]
enum Scan {
    Window { attempt: u64, turn: u32, after: Option<u64>, checked: bool },
    Turn { attempt: u64, turn: u32 },
    Page,
}
#[derive(Debug)]
enum Step {
    Waiting,
    Opening,
    Evicting { task: u64 },
    Length { offset: u64, bytes: Writer },
    Frame { offset: u64, length: u32, bytes: Writer },
}
#[derive(Debug)]
pub(crate) struct Read {
    task: u64,
    end: u64,
    position: u64,
    backwards: bool,
    scan: Scan,
    step: Step,
    turns: List<TurnRecord>,
    budget: u64,
}
pub(crate) fn worst_case(limits: &Limits, codec: &codec::Limits) -> Option<u64> {
    let rows = limits.store.page.rows;
    let turn = crate::core_v1::TurnRecord::worst_case_heap(&codec.core)?;
    let frame = crate::core_v1::TurnRecord::worst_case_bytes(&codec.core)?.checked_add(12)?;
    Map::<Token, Read>::worst_case(limits.reads)?
        .checked_add(
            List::<TurnRecord>::worst_case(rows)?
                .checked_add(turn.checked_mul(u64::from(rows))?)?
                .checked_mul(u64::from(limits.reads))?,
        )?
        .checked_add(frame.checked_mul(3)?)
}
pub(crate) fn transcript(
    store: &mut Store,
    owner: Token,
    read: TranscriptRead,
    rows: u32,
    bytes: u64,
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    assert!(!store.transcripts.reads.contains_key(&owner), "one transcript read per owner");
    assert!(store.transcripts.reads.len() < store.limits.reads, "startup bounds cover transcript reads");
    let Some(read) = prepare(store, owner, read, rows, bytes) else {
        above.push(Event::Unread { owner });
        return;
    };
    store.transcripts.reads.insert(owner, read).expect("read slot checked at admission");
    fire(store, above, below);
}
fn prepare(store: &mut Store, owner: Token, request: TranscriptRead, rows: u32, bytes: u64) -> Option<Read> {
    match store.phase {
        Phase::Ready { .. } => {}
        Phase::Closed | Phase::Stopped => return None,
    }
    let task = match request {
        TranscriptRead::Window { task, .. } | TranscriptRead::Turn { task, .. } | TranscriptRead::Page { task, .. } => {
            task
        }
    };
    let key = keys::encode_own(&keys::Own::Transcript { task }, store.limits.store.key)?;
    let row = open::get(store, owner, key).ok()??;
    let place = codec::component::Transcript::decode(&row).ok()?;
    let (position, backwards, scan) = match request {
        TranscriptRead::Window { attempt, turn, after, .. } => {
            if place.window >= place.length
                || after.unwrap_or(place.window) < place.window
                || after.unwrap_or(place.window) > place.length
            {
                return None;
            }
            (place.window, false, Scan::Window { attempt, turn, after, checked: false })
        }
        TranscriptRead::Turn { attempt, turn, .. } => {
            let key = keys::encode_own(&keys::Own::Attempt { task, attempt }, store.limits.store.key)?;
            let row = open::get(store, owner, key).ok()??;
            let offset = codec::component::Attempt::decode(&row).ok()?.first;
            if offset >= place.length {
                return None;
            }
            (offset, false, Scan::Turn { attempt, turn })
        }
        TranscriptRead::Page { backwards, after, .. } => {
            let offset = after.unwrap_or(if backwards { place.length } else { 0 });
            if offset > place.length {
                return None;
            }
            (offset, backwards, Scan::Page)
        }
    };
    Some(Read {
        task,
        end: place.length,
        position,
        backwards,
        scan,
        step: Step::Waiting,
        turns: List::with_capacity(rows.min(store.limits.store.page.rows)),
        budget: bytes.min(u64::from(store.limits.store.page.bytes)),
    })
}
pub(crate) fn fire(store: &mut Store, above: &mut Queue<Event>, below: &mut Below) {
    if store.transcripts.reading.is_some() {
        return;
    }
    match store.transcripts.stage {
        Stage::Idle => {}
        Stage::Opening
        | Stage::Evicting { .. }
        | Stage::Making
        | Stage::RootSync
        | Stage::DirectoryOpen
        | Stage::Creating
        | Stage::Writing
        | Stage::Syncing
        | Stage::DirectorySync
        | Stage::DirectoryClose
        | Stage::Dropping => return,
    }
    let owner = match store.transcripts.reads.first() {
        Some((owner, _)) => *owner,
        None => return,
    };
    let mut read = store.transcripts.reads.remove(&owner).expect("waiting reader");
    store.transcripts.reading = Some(owner);
    let ready = if store.transcripts.files.contains_key(&read.task) {
        next(store, &mut read, below)
    } else {
        if store.transcripts.files.len() == store.limits.files {
            let mut oldest = None;
            for (task, file) in &store.transcripts.files {
                if let Some((_, touched)) = oldest
                    && file.touched >= touched
                {
                    continue;
                }
                oldest = Some((*task, file.touched));
            }
            let (task, _) = oldest.expect("a full nonempty file set");
            read.step = Step::Evicting { task };
            let file = store.transcripts.files.get(&task).expect("oldest file").token;
            below.transcripts.push(file::Request::Close { owner: OWNER, file });
        } else {
            opening(store, &mut read, below);
        }
        Ok(false)
    };
    settle(store, owner, read, ready, above);
}
fn opening(store: &Store, read: &mut Read, below: &mut Below) {
    read.step = Step::Opening;
    below.transcripts.push(file::Request::OpenRead {
        owner: OWNER,
        root: store.directory.transcripts.expect("opened transcripts root").fd,
        name: super::path(read.task),
    });
}
fn next(store: &mut Store, read: &mut Read, below: &mut Below) -> Result<bool, ()> {
    if read.position == if read.backwards { 0 } else { read.end } {
        return Ok(true);
    }
    let checking_window = match read.scan {
        Scan::Window { checked, .. } => !checked,
        Scan::Turn { .. } | Scan::Page => false,
    };
    if read.turns.room() == 0 && !checking_window {
        return Ok(true);
    }
    let offset = if read.backwards { read.position.checked_sub(4).ok_or(())? } else { read.position };
    if offset.checked_add(4).ok_or(())? > read.end {
        return Err(());
    }
    read.step = Step::Length { offset, bytes: Writer::new(4) };
    demand(store, read, offset, 4, below)?;
    Ok(false)
}
fn demand(store: &mut Store, read: &Read, offset: u64, max: u32, below: &mut Below) -> Result<(), ()> {
    let touch = store.transcripts.touch.checked_add(1).ok_or(())?;
    store.transcripts.touch = touch;
    let file = store.transcripts.files.get_mut(&read.task).ok_or(())?;
    file.touched = touch;
    below.transcripts.push(file::Request::ReadAt { owner: OWNER, file: file.token, offset, max });
    Ok(())
}
pub(crate) fn arrived(store: &mut Store, arrived: Arrived, above: &mut Queue<Event>, below: &mut Below) {
    let Some(owner) = store.transcripts.reading else { return };
    let mut read = store.transcripts.reads.remove(&owner).expect("active reader");
    let result = match arrival(arrived.event) {
        Some(event) => advance(store, &mut read, event, below),
        None => Err(()),
    };
    settle(store, owner, read, result, above);
}
fn advance(store: &mut Store, read: &mut Read, event: Arrival, below: &mut Below) -> Result<bool, ()> {
    let step = core::mem::replace(&mut read.step, Step::Waiting);
    match step {
        Step::Opening => {
            let file = match event {
                Arrival::Opened(file) => file,
                Arrival::Closed | Arrival::Read(_) => return Err(()),
            };
            store
                .transcripts
                .files
                .insert(read.task, File { token: file, touched: store.transcripts.touch, writable: false })
                .expect("bounded file slot after open");
            next(store, read, below)
        }
        Step::Evicting { task } => {
            match event {
                Arrival::Closed => {}
                Arrival::Opened(_) | Arrival::Read(_) => return Err(()),
            }
            let _: Option<File> = store.transcripts.files.remove(&task);
            opening(store, read, below);
            Ok(false)
        }
        Step::Length { offset, mut bytes } => {
            if !append(event, &mut bytes)? {
                let consumed = u64::try_from(bytes.written()).expect("four byte length");
                let max = u32::try_from(bytes.room()).expect("four byte length");
                demand(store, read, offset.checked_add(consumed).ok_or(())?, max, below)?;
                read.step = Step::Length { offset, bytes };
                return Ok(false);
            }
            let length = Reader::new(&bytes.finish()).u32().ok_or(())?;
            let max =
                crate::core_v1::TurnRecord::worst_case_bytes(&store.codec.core).ok_or(())?.checked_add(12).ok_or(())?;
            if length < 12 || u64::from(length) > max {
                return Err(());
            }
            let start =
                if read.backwards { read.position.checked_sub(u64::from(length)).ok_or(())? } else { read.position };
            if start.checked_add(u64::from(length)).ok_or(())? > read.end {
                return Err(());
            }
            read.step =
                Step::Frame { offset: start, length, bytes: Writer::new(usize::try_from(length).expect("u32 frame")) };
            demand(store, read, start, length, below)?;
            Ok(false)
        }
        Step::Frame { offset, length, mut bytes } => {
            if !append(event, &mut bytes)? {
                let consumed = u64::try_from(bytes.written()).expect("bounded frame");
                let max = u32::try_from(bytes.room()).expect("bounded frame");
                demand(store, read, offset.checked_add(consumed).ok_or(())?, max, below)?;
                read.step = Step::Frame { offset, length, bytes };
                return Ok(false);
            }
            let Ok(turn) = frame::decode(&bytes.finish(), &store.codec) else {
                return Err(());
            };
            accept(store, read, offset, length, turn, below)
        }
        Step::Waiting => Err(()),
    }
}
fn append(event: Arrival, buffer: &mut Writer) -> Result<bool, ()> {
    let bytes = match event {
        Arrival::Read(bytes) => bytes,
        Arrival::Opened(_) | Arrival::Closed => return Err(()),
    };
    if bytes.is_empty() {
        return Err(());
    }
    if buffer.put(&bytes).is_err() {
        return Err(());
    }
    Ok(buffer.room() == 0)
}
fn accept(
    store: &mut Store,
    read: &mut Read,
    offset: u64,
    length: u32,
    turn: TurnRecord,
    below: &mut Below,
) -> Result<bool, ()> {
    if turn.task != read.task {
        return Err(());
    }
    let found = match &mut read.scan {
        Scan::Window { attempt, turn: number, after, checked } => {
            if !*checked {
                if turn.attempt != *attempt || turn.turn != *number || !turn.opens_window {
                    return Err(());
                }
                *checked = true;
                if let Some(after) = *after
                    && after != offset
                {
                    read.position = after;
                    return next(store, read, below);
                }
            }
            true
        }
        Scan::Turn { attempt, turn: number } => {
            if turn.attempt != *attempt || turn.turn > *number {
                return Err(());
            }
            turn.turn == *number
        }
        Scan::Page => true,
    };
    if found {
        if read.turns.room() == 0 || u64::from(length) > read.budget {
            return Ok(true);
        }
        read.budget = read.budget.checked_sub(u64::from(length)).expect("frame within budget");
        read.turns.push(turn).expect("next checked row capacity");
    }
    read.position = if read.backwards { offset } else { offset.checked_add(u64::from(length)).ok_or(())? };
    match read.scan {
        Scan::Turn { .. } => {
            if found {
                Ok(true)
            } else if read.position == read.end {
                Err(())
            } else {
                next(store, read, below)
            }
        }
        Scan::Window { .. } | Scan::Page => next(store, read, below),
    }
}
fn settle(store: &mut Store, owner: Token, read: Read, result: Result<bool, ()>, above: &mut Queue<Event>) {
    match result {
        Ok(false) => {
            store.transcripts.reads.insert(owner, read).expect("active read retains its slot");
        }
        Ok(true) => {
            let end = if read.backwards { 0 } else { read.end };
            let next = match read.scan {
                Scan::Turn { .. } => None,
                Scan::Window { .. } | Scan::Page => {
                    if read.position == end {
                        None
                    } else {
                        Some(read.position)
                    }
                }
            };
            let mut turns = read.turns.into_boxed();
            if read.backwards {
                turns.reverse();
            }
            store.transcripts.reading = None;
            above.push(Event::Turns { owner, turns, next });
        }
        Err(()) => {
            store.transcripts.reading = None;
            above.push(Event::Unread { owner });
        }
    }
}
enum Arrival {
    Opened(Token),
    Closed,
    Read(Box<[u8]>),
}
fn arrival(event: file::Event) -> Option<Arrival> {
    match event {
        file::Event::Opened { file, .. } => Some(Arrival::Opened(file)),
        file::Event::Closed { .. } => Some(Arrival::Closed),
        file::Event::Read { bytes, .. } => Some(Arrival::Read(bytes)),
        file::Event::Made { .. }
        | file::Event::Refused { .. }
        | file::Event::Loaded { .. }
        | file::Event::Scanned { .. }
        | file::Event::Stored { .. }
        | file::Event::Conflict { .. }
        | file::Event::TooLarge { .. }
        | file::Event::Stated { .. }
        | file::Event::Written { .. }
        | file::Event::Cut { .. }
        | file::Event::Synced { .. }
        | file::Event::Renamed { .. }
        | file::Event::Removed { .. }
        | file::Event::Listed { .. }
        | file::Event::Failed { .. }
        | file::Event::Cancelled { .. } => None,
    }
}
