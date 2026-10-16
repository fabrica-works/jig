use super::*;
const PARENT: u64 = 1;
const CHILD: u64 = 7;
const CHARTER: u32 = 1;
const ATTEMPT: u64 = 1;
const DELEGATES: [(tasks::Phase, &[u8]); 2] = [
    (
        tasks::Phase::Active(tasks::Active::Running { attempt: ATTEMPT }),
        b"delegate 7: running, spec revision 1\n".as_slice(),
    ),
    (
        tasks::Phase::Held { was: tasks::Was::Waiting, why: tasks::Hold::Budget, source: None },
        b"delegate 7: held, spec revision 1, hold revision 2\n".as_slice(),
    ),
];
#[test]
fn the_briefs_delegates_show_their_spec_and_hold_revisions() {
    let (mut core, _) = make_tree();
    for (phase, expected) in DELEGATES {
        let mut context = charters::context(PARENT, CHARTER);
        context.delegates = Box::new([tasks::DelegateState {
            task: CHILD,
            phase,
            revisions: tasks::Revisions { spec: 1, hold: 2, person: 0 },
        }]);
        core.contexts.insert(PARENT, context).expect("context room");
        let text = read_brief_part(
            &core,
            BriefTextLimits { parts: 1, read_bytes: 256, brief_bytes: 256 },
            PARENT,
            BriefPart::Delegates,
        )
        .expect("delegates section");
        assert_eq!(text.as_ref(), expected);
    }
}
