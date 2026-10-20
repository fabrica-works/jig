//! Different receiving bounds in one shell loop (client/domain/testing.md, 4).
use crate::{Scripted, Settings, Test, World, scripted::Family, test};
use jig_client_domain::contract::{DocumentText, ObjectKey, ReadValue, Text, VersionRange};
use jig_client_scripted_engine::{Bounds, Script, Truth};
use skein_lib::{Duration, Wall};

/// A supplied frame name past the default 32-byte receiving bound.
pub const NAME: &[u8] = b"Pat with a supplied name longer than the default receiving bound";
/// The root, view and callback limits declared by this fixture.
#[must_use]
pub fn limits() -> jig_client_test_domain::Limits {
    let mut limits = test::limits();
    limits.jig.text = 128;
    limits.first.text = 128;
    limits
}
/// Constructs both sides of the client with the fixture's declared bounds.
#[must_use]
pub fn client(seed: u64) -> (jig_client_test_domain::Domain, jig_client_test_view::View<'static>) {
    test::new(&limits(), seed)
}
/// Supplies one wide frame and one ordinary frame to independent clients.
#[must_use]
pub fn world(settings: Settings) -> World<Test, Scripted> {
    let mut truth = Truth::default();
    let mut wide = super::frame(7);
    wide.party.name = DocumentText::complete(Text::new(NAME.into(), 128).expect("frame name fits"));
    truth.frames.insert(7, wide);
    truth.frames.insert(8, super::frame(8));
    truth.sign_ins.insert(7, 7);
    truth.sign_ins.insert(8, 8);
    let seed = settings.seed;
    let mut world = World::new(
        settings,
        Scripted::new(
            Script {
                versions: VersionRange { first: 10, last: 10 },
                truth,
                bounds: Bounds {
                    text: 128,
                    rows: 8,
                    authority: 8,
                    segments: 4,
                    document_bytes: 65536,
                    commit_delay: Duration::from_millis(5),
                    heartbeat: Duration::from_millis(20),
                    watches: 64,
                    watches_per_party: 32,
                },
                rules: vec![],
            },
            seed,
            Family::default(),
        ),
    );
    let now = world.now();
    world
        .front()
        .engine
        .act(
            now,
            Wall::EPOCH,
            jig_client_scripted_engine::Change::Object {
                project: 1,
                key: ObjectKey::Task(9),
                value: ReadValue::Task(Box::new(super::task())),
            },
        )
        .expect("starting task fits");
    world
}
