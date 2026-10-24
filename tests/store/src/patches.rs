//! The two fixture binaries and their independent record model (store.md 14).
use crate::{World, root};
use jig_protocol_store::{KeptFamily, Keys, Owner, PatchSet, PatchSets, Write};
use jig_test_patches::{self as fixture, patches as app};
use jig_test_patches_jig::{self as upstream, patches as jig};
use std::collections::BTreeMap;
const LABEL: KeptFamily = KeptFamily {
    name: "fixture-label",
    texts: app::kept::fixture_label::TEXTS,
    keys: &[Keys { from: fixture::keys::LABELS_FROM, to: fixture::keys::LABELS_TO, root: fixture::keys::LABELS_ROOT }],
};
const SHELF: KeptFamily = KeptFamily {
    name: "fixture-shelf",
    texts: app::kept::fixture_shelf::TEXTS,
    keys: &[Keys {
        from: fixture::keys::SHELVES_FROM,
        to: fixture::keys::SHELVES_TO,
        root: fixture::keys::SHELVES_ROOT,
    }],
};
const NOTE: KeptFamily = KeptFamily {
    name: "fixture-note",
    texts: jig::kept::fixture_note::TEXTS,
    keys: &[Keys { from: upstream::keys::NOTES_FROM, to: upstream::keys::NOTES_TO, root: upstream::keys::NOTES_ROOT }],
};
const APPLICATION: PatchSet =
    PatchSet { owner: Owner::Application, name: fixture::NAME, families: &[LABEL, SHELF], patches: app::PATCHES };
const JIG: PatchSet = PatchSet { owner: Owner::Jig, name: upstream::NAME, families: &[NOTE], patches: jig::PATCHES };
pub const SETS: PatchSets = PatchSets { sets: &[APPLICATION, JIG] };
pub const BASE: PatchSets = PatchSets {
    sets: &[
        PatchSet {
            families: &[
                KeptFamily { texts: &[app::kept::fixture_label::TEXTS[0]], ..LABEL },
                KeptFamily { texts: &[app::kept::fixture_shelf::TEXTS[0]], ..SHELF },
            ],
            patches: &[],
            ..APPLICATION
        },
        PatchSet {
            families: &[KeptFamily { texts: &[jig::kept::fixture_note::TEXTS[0]], ..NOTE }],
            patches: &[],
            ..JIG
        },
    ],
};

/// Fixture rows need 64 bytes; 1024 also leaves room for the component's
/// format, layouts and deployment header. Two-row walks exercise paging.
#[must_use]
pub fn limits() -> jig_protocol_store::Limits {
    let mut limits = root::limits();
    limits.journal.writes = 32;
    for map in [&mut limits.store, &mut limits.secrets] {
        map.key = 64;
        map.value = 1024;
        map.ops = 64;
        map.commit = 4096;
        map.queued_bytes = 8192;
        map.chunk = 4096;
        map.page.bytes = 2048;
    }
    limits
}

/// Two labels, three shelves and two notes cross two-record page boundaries.
pub const STORY_LABELS: u32 = 2;
pub const STORY_SHELVES: u32 = 3;
pub const STORY_NOTES: u32 = 2;

#[derive(Clone, Debug)]
pub struct Shelf {
    pub key: Vec<u8>,
    pub name: Vec<u8>,
    pub books: u32,
    pub caption: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Records {
    pub labels: Vec<(Vec<u8>, Vec<u8>)>,
    pub shelves: Vec<Shelf>,
    pub notes: Vec<(Vec<u8>, Vec<u8>)>,
}

fn text(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend(u32::try_from(bytes.len()).expect("fixture text length").to_be_bytes());
    out.extend(bytes);
}
fn label(caption: &[u8], version: u16) -> Vec<u8> {
    let mut out = version.to_be_bytes().to_vec();
    text(&mut out, caption);
    if version == 2 {
        out.push(0);
    }
    out
}

impl Records {
    #[must_use]
    pub fn story() -> Self {
        Self {
            labels: (0..STORY_LABELS).map(|n| (format!("l/{n}").into_bytes(), b"caption".to_vec())).collect(),
            shelves: (0..STORY_SHELVES)
                .map(|n| Shelf {
                    key: format!("s/{n}").into_bytes(),
                    name: b"shelf".to_vec(),
                    books: n + 1,
                    caption: b"held".to_vec(),
                })
                .collect(),
            notes: (0..STORY_NOTES).map(|n| (format!("n/{n}").into_bytes(), b"note".to_vec())).collect(),
        }
    }

    /// Independent wire model: no patch function computes expected bytes.
    #[must_use]
    pub fn rows(&self, version: u16) -> BTreeMap<Box<[u8]>, Box<[u8]>> {
        let mut rows = BTreeMap::new();
        for (key, caption) in &self.labels {
            rows.insert(key.clone().into_boxed_slice(), label(caption, version).into_boxed_slice());
        }
        for Shelf { key, name, books, caption } in &self.shelves {
            let mut out = version.to_be_bytes().to_vec();
            text(&mut out, name);
            out.extend(books.to_be_bytes());
            text(&mut out, &label(caption, version));
            if version == 2 {
                out.extend((books * 10 + u32::try_from(key.len()).expect("relative key length")).to_be_bytes());
            }
            rows.insert(key.clone().into_boxed_slice(), out.into_boxed_slice());
        }
        for (key, content) in &self.notes {
            let mut out = version.to_be_bytes().to_vec();
            text(&mut out, content);
            if version == 2 {
                out.push(0);
            }
            rows.insert(key.clone().into_boxed_slice(), out.into_boxed_slice());
        }
        rows
    }

    #[must_use]
    pub fn writes(&self, version: u16) -> Box<[Write]> {
        let mut writes = vec![Write::Save(root::header(1))];
        writes.extend(self.rows(version).into_iter().map(|(key, value)| Write::Put { key, value }));
        writes.into_boxed_slice()
    }

    #[must_use]
    pub fn committed(&self, seed: u64, sets: PatchSets) -> World {
        self.committed_at(seed, sets, 1)
    }

    #[must_use]
    pub fn committed_at(&self, seed: u64, sets: PatchSets, version: u16) -> World {
        let mut world = World::unopened_observed(seed, limits(), None);
        world.patch_sets(sets);
        assert_eq!(world.open(), Some(0));
        assert_eq!(world.commit(self.writes(version)), Some(1));
        world.crash(seed);
        world
    }

    /// Decode by each family's kept text as well as compare every modeled byte.
    pub fn check(&self, world: &mut World, version: u16, context: &dyn std::fmt::Debug) {
        let rows = world.application(false);
        assert_eq!(rows, self.rows(version), "all records belong to one whole level: {context:?}");
        for (key, bytes) in rows {
            let (texts, root) = match key[0] {
                b'l' => (app::kept::fixture_label::TEXTS, "Label"),
                b's' => (app::kept::fixture_shelf::TEXTS, "Shelf"),
                b'n' => (jig::kept::fixture_note::TEXTS, "Note"),
                _ => panic!("fixture key"),
            };
            let schema = skein_codec::schema::parse(texts[usize::from(version - 1)].as_bytes()).expect("kept schema");
            let root = schema.find(root.as_bytes()).expect("record root");
            let mut value = skein_codec::value::Value::with_room(16);
            value.decode(&schema, root, bytes).expect("record decodes at its level");
        }
    }
}

/// All mutations from the copy's first mkdir through active's final sync.
/// The last cut also covers power loss after the switch's rename.
/// The ordinals name kernel calls, including durable replace's internals.
#[must_use]
pub fn cuts(world: &World) -> Vec<u32> {
    use crate::world::Change;
    let changes = world.changes();
    let first = changes.iter().position(|(_, change)| *change == Change::Make).expect("copy mkdir");
    let last = changes.iter().rposition(|(_, change)| *change == Change::Rename).expect("switch rename");
    assert!(first < last);
    assert_eq!(changes[last + 1].1, Change::Sync, "switch directory sync follows its rename");
    changes[first..=last + 1].iter().map(|(operation, _)| *operation).collect()
}

pub fn whole(world: &mut World) {
    assert_eq!(world.active(), b"2\n");
    let paths = world.paths();
    let generations: Vec<_> = paths
        .iter()
        .filter(|path| path.starts_with(b"generations/") && skein_lib::bytes::count(path, b"/", 2) == 1)
        .map(Vec::as_slice)
        .collect();
    assert_eq!(generations, [b"generations/1".as_slice(), b"generations/2".as_slice()]);
    assert!(paths.iter().any(|path| path == b"generations/2/manifest"));
    assert!(!paths.iter().any(|path| path.windows(7).any(|part| part == b".skein-")));
}

/// The previous generation is an opaque, byte-for-byte rollback image.
#[must_use]
pub fn source(world: &mut World) -> BTreeMap<Vec<u8>, Option<Vec<u8>>> {
    world.contents().into_iter().filter(|(path, _)| path.starts_with(b"generations/1/")).collect()
}
