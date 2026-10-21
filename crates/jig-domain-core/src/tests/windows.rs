use super::*;

fn window_core(resume_bytes: u32) -> Core {
    let mut config = config(1);
    config.settings.resume_bytes = resume_bytes;
    let mut core = Core::new(config, &limits()).expect("fixture core configuration");
    core.begin_transcript(1, 1);
    window(&mut core).window = Some((1, 1));
    core
}

fn window(core: &mut Core) -> &mut Transcript {
    core.transcripts.get_mut(&1).expect("begun window")
}

fn resumed(core: &mut Core) -> Box<[Box<[u8]>]> {
    let transcript = core.transcripts.remove(&1).expect("loaded window");
    core.resumed_turns(transcript)
}

fn append(core: &mut Core, number: u32, body: &[u8]) {
    assert!(core.append_transcript(
        1,
        TurnRecord {
            task: 1,
            attempt: 1,
            turn: number,
            spent: 0,
            read: None,
            opens_window: number == 1,
            at: Wall::EPOCH,
            transcript: body.into(),
        },
    ));
}

#[test]
fn many_one_byte_turns_start_fresh_when_their_lengths_pass_the_limit() {
    let mut core = window_core(200);
    for number in 1..=40 {
        append(&mut core, number, b"a");
    }
    assert!(!core.transcript_oversized(1));
    assert_eq!(window(&mut core).turns.len(), 40);
    append(&mut core, 41, b"b");
    assert!(core.transcript_oversized(1));
    assert_eq!(window(&mut core).turns.len(), 40);
    assert!(resumed(&mut core).is_empty());
}

#[test]
fn empty_turns_count_their_lengths_and_resume_in_order() {
    let mut core = window_core(9);
    append(&mut core, 1, b"");
    append(&mut core, 2, b"a");
    assert_eq!(resumed(&mut core).as_ref(), [Box::<[u8]>::from(&b""[..]), Box::from(&b"a"[..])]);
}

#[test]
fn an_oversized_turn_keeps_a_tail_with_room_for_its_length() {
    let mut core = window_core(8);
    append(&mut core, 1, b"abcdefgh");
    let tail = read_brief_part(
        &core,
        BriefTextLimits { parts: 1, read_bytes: 64, brief_bytes: 64 },
        1,
        BriefPart::TranscriptTail,
    )
    .expect("window tail");
    assert_eq!(tail.as_ref(), b"[8 bytes cut]\nefgh");
}

#[test]
fn a_limit_smaller_than_a_length_retains_no_turn() {
    let mut core = window_core(3);
    append(&mut core, 1, b"");
    assert!(core.transcript_oversized(1));
    assert!(window(&mut core).turns.is_empty());
}
