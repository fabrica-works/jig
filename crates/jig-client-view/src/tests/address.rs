use crate::address::{self, Address, Codec, Names, Place, ProjectSection, TaskSection};
use skein_lib::{Rng, Writer};

const TASK: u64 = 7;
const PROJECT: u32 = 4;
const MAX_TASK: u64 = u64::MAX;
const MAX_PROJECT: u32 = u32::MAX;
const PREFIX: &[u8] = b"/work/ui";
const PRINTED: &[(Address, &[u8])] = &[
    (Address::Inbox { project: None }, b"/inbox"),
    (Address::Inbox { project: Some(PROJECT) }, b"/inbox/4"),
    (Address::Conversations { project: None }, b"/conversations"),
    (Address::Conversations { project: Some(PROJECT) }, b"/conversations/4"),
    (Address::Task { task: MAX_TASK, section: TaskSection::Overview }, b"/tasks/18446744073709551615"),
    (Address::Task { task: TASK, section: TaskSection::Activity }, b"/tasks/7/activity"),
    (Address::Task { task: TASK, section: TaskSection::Plan }, b"/tasks/7/plan"),
    (Address::Task { task: TASK, section: TaskSection::History }, b"/tasks/7/history"),
    (Address::Task { task: TASK, section: TaskSection::Budget }, b"/tasks/7/budget"),
    (Address::Task { task: TASK, section: TaskSection::Panels }, b"/tasks/7/panels"),
    (Address::Project { project: MAX_PROJECT, section: ProjectSection::Overview }, b"/projects/4294967295"),
    (Address::Project { project: PROJECT, section: ProjectSection::Goals }, b"/projects/4/goals"),
    (Address::Project { project: PROJECT, section: ProjectSection::Tasks }, b"/projects/4/tasks"),
    (Address::Project { project: PROJECT, section: ProjectSection::Notes }, b"/projects/4/notes"),
    (Address::Project { project: PROJECT, section: ProjectSection::Settings }, b"/projects/4/settings"),
    (Address::Project { project: PROJECT, section: ProjectSection::Resources }, b"/projects/4/resources"),
    (Address::Project { project: PROJECT, section: ProjectSection::Policy }, b"/projects/4/policy"),
    (Address::Project { project: PROJECT, section: ProjectSection::Roles }, b"/projects/4/roles"),
    (Address::Project { project: PROJECT, section: ProjectSection::Spend }, b"/projects/4/spend"),
    (Address::Project { project: PROJECT, section: ProjectSection::Recurring }, b"/projects/4/recurring"),
    (Address::Deployment, b"/deployment"),
];
const OTHER_PRINTED: &[&[u8]] = &[
    b"/queue",
    b"/queue/4",
    b"/chats",
    b"/chats/4",
    b"/items/18446744073709551615",
    b"/items/7/a",
    b"/items/7/p",
    b"/items/7/h",
    b"/items/7/b",
    b"/items/7/c",
    b"/groups/4294967295",
    b"/groups/4/g",
    b"/groups/4/t",
    b"/groups/4/n",
    b"/groups/4/s",
    b"/groups/4/r",
    b"/groups/4/p",
    b"/groups/4/o",
    b"/groups/4/b",
    b"/groups/4/c",
    b"/server",
];
const INVALID: &[&[u8]] = &[
    b"/inbox/04",
    b"/conversations/4294967296",
    b"/tasks/07",
    b"/tasks/18446744073709551616",
    b"/projects/4294967296",
    b"/projects/+4",
    b"/projects/-1",
    b"/tasks/7/overview",
    b"/projects/4/overview",
    b"/tasks",
    b"/projects",
    b"/deployment/4",
    b"/inbox/4/extra",
    b"/inbox//",
    b"//",
    b"/tasks/7//",
    b"/elsewhere",
    b"/tasks/7/budget/more",
    b"/projects/4/settings/more",
];

#[test]
fn each_place_prints_its_path_and_parses_back_at_the_root() {
    for (place, path) in PRINTED {
        assert_eq!(&*address::print(&address::REFERENCE, *place), *path, "{place:?}");
        assert_eq!(address::parse(&address::REFERENCE, path), Place::Jig(*place), "{path:?}");
    }
}

#[test]
fn each_place_under_a_prefix_prints_and_parses_back() {
    let codec = Codec { prefix: PREFIX, ..address::REFERENCE };
    for (place, path) in PRINTED {
        let prefixed = joined(PREFIX, path);
        assert_eq!(address::print(&codec, *place), prefixed, "{place:?}");
        assert_eq!(address::parse(&codec, &prefixed), Place::Jig(*place), "{path:?}");
    }
}

#[test]
fn a_path_outside_the_prefix_is_the_applications() {
    let codec = Codec { prefix: PREFIX, ..address::REFERENCE };
    for path in [b"/work/uix/inbox".as_slice(), b"/inbox", b"/", b"/work"] {
        assert_eq!(address::parse(&codec, path), Place::Elsewhere, "{path:?}");
    }
}

#[test]
fn other_segment_names_print_and_parse_their_own_paths() {
    let codec = Codec {
        prefix: b"",
        names: Names {
            inbox: b"queue",
            conversations: b"chats",
            tasks: b"items",
            projects: b"groups",
            deployment: b"server",
            task_sections: [b"a", b"p", b"h", b"b", b"c"],
            project_sections: [b"g", b"t", b"n", b"s", b"r", b"p", b"o", b"b", b"c"],
        },
    };
    assert_eq!(PRINTED.len(), OTHER_PRINTED.len());
    for ((place, _), path) in PRINTED.iter().zip(OTHER_PRINTED) {
        assert_eq!(&*address::print(&codec, *place), *path, "{place:?}");
        assert_eq!(address::parse(&codec, path), Place::Jig(*place), "{place:?}");
    }
    assert_eq!(&*address::print(&codec, Address::task(TASK)), b"/items/7");
    assert_eq!(address::parse(&codec, b"/inbox"), Place::Missing);
}

#[test]
fn a_query_and_one_trailing_slash_are_ignored() {
    for (place, path) in PRINTED {
        for suffix in [b"/".as_slice(), b"?filter=1", b"/?filter=1"] {
            assert_eq!(address::parse(&address::REFERENCE, &joined(path, suffix)), Place::Jig(*place), "{path:?}");
        }
    }
}

#[test]
fn the_root_and_prefix_alone_are_the_home() {
    let codec = Codec { prefix: PREFIX, ..address::REFERENCE };
    for path in [PREFIX, b"/work/ui/", b"/work/ui?filter=1"] {
        assert_eq!(address::parse(&codec, path), Place::Jig(Address::Inbox { project: None }), "{path:?}");
    }
    assert_eq!(address::parse(&address::REFERENCE, b"/"), Place::Jig(Address::Inbox { project: None }));
}

#[test]
fn a_leading_zero_or_a_number_past_its_type_is_missing() {
    for path in INVALID {
        assert_eq!(address::parse(&address::REFERENCE, path), Place::Missing, "{path:?}");
    }
}

#[test]
fn random_bytes_parse_to_a_place_without_a_panic() {
    let mut rng = Rng::new(7);
    for _ in 0..2_000_u32 {
        let mut path = [0; 64];
        for byte in &mut path {
            *byte = u8::try_from(rng.next_u64() & 0xff).unwrap();
        }
        let length = usize::try_from(rng.below(65)).unwrap();
        let _place = address::parse(&address::REFERENCE, &path[..length]);
    }
}

fn joined(first: &[u8], second: &[u8]) -> Box<[u8]> {
    let mut bytes = Writer::new(first.len().checked_add(second.len()).unwrap());
    bytes.put(first).unwrap();
    bytes.put(second).unwrap();
    bytes.finish()
}
