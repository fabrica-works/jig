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
        Some(body.into()),
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
fn an_oversized_resume_turn_keeps_its_text_independently() {
    let mut core = window_core(8);
    append(&mut core, 1, b"abcdefgh");
    let tail = read_brief_part(
        &core,
        BriefTextLimits { parts: 1, read_bytes: 64, brief_bytes: 64 },
        1,
        BriefPart::TranscriptTail,
    )
    .expect("window tail");
    assert_eq!(tail.as_ref(), b"abcdefgh");
}

#[test]
fn a_limit_smaller_than_a_length_retains_no_turn() {
    let mut core = window_core(3);
    append(&mut core, 1, b"");
    assert!(core.transcript_oversized(1));
    assert!(window(&mut core).turns.is_empty());
}

fn tail(core: &Core, bytes: u32) -> Box<[u8]> {
    read_brief_part(
        core,
        BriefTextLimits { parts: 1, read_bytes: bytes, brief_bytes: 256 },
        1,
        BriefPart::TranscriptTail,
    )
    .expect("rendered tail")
}

#[test]
fn a_fresh_brief_keeps_whole_newest_texts_under_its_own_budget() {
    let (mut core, mut limits) = make_tree();
    limits.brief_core_budgets.tail = 90;
    limits.brief_core_budgets.task = 128;
    core.settings.tail_bytes = 90;
    core.settings.charters.get_mut(0).unwrap().policy.resume = false;
    send_task(&mut core, &limits, tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(1)), task: 1 });
    let context = core.tasks.run_context(1).unwrap();
    core.contexts.insert(1, context).unwrap();
    core.begin_transcript(1, 1);
    window(&mut core).window = Some((1, 1));
    for (number, body) in [(1, [b'a'; 40]), (2, [b'b'; 40]), (3, [b'c'; 40])] {
        append(&mut core, number, &body);
    }
    let mut expected = [b'c'; 80];
    expected.get_mut(..40).unwrap().fill(b'b');
    assert_eq!(tail(&core, 90).as_ref(), expected);
    let out = watch_outputs(&mut core, &limits, Event::BriefNotes { task: 1, lines: List::with_capacity(0), more: 0 });
    let mut planned = false;
    for request in out {
        if let Request::Now(now) = request
            && let Now::BriefCorePlanned { sections, .. } = *now
        {
            for section in &sections {
                if let jig_domain_brief::Planned::Core {
                    kind: jig_domain_brief::Core::TranscriptTail,
                    text,
                    limit,
                    ..
                } = section
                {
                    assert_eq!(*limit, 90);
                    assert_eq!(text.as_ref(), expected);
                    planned = true;
                }
            }
        }
    }
    assert!(planned, "fresh brief plans its text tail");
}

#[test]
fn a_missing_rendering_is_a_readable_line() {
    let mut core = window_core(256);
    assert!(core.append_transcript(
        1,
        TurnRecord {
            task: 1,
            attempt: 1,
            turn: 1,
            spent: 0,
            read: None,
            opens_window: true,
            at: Wall::EPOCH,
            transcript: Box::from(&[0xff, 0x80][..]),
        },
        None
    ));
    assert_eq!(tail(&core, 128).as_ref(), b"[a turn not readable as text]\n");
}

#[test]
fn an_oversized_text_and_a_smaller_read_keep_utf8_suffixes_within_the_bound() {
    let mut core = window_core(256);
    core.settings.tail_bytes = 26;
    // The 7-byte suffix starts in the middle of the first é.
    append(&mut core, 1, "some earlier words éééé".as_bytes());
    let text = tail(&core, 128);
    assert_eq!(text.as_ref(), "[earlier text cut]\nééé".as_bytes());
    assert!(text.len() <= 26);
    let text = tail(&core, 24);
    assert_eq!(text.as_ref(), "[earlier text cut]\néé".as_bytes());
    assert!(text.len() <= 24);
    assert_eq!(resumed(&mut core).as_ref(), [Box::<[u8]>::from("some earlier words éééé".as_bytes())]);
}
