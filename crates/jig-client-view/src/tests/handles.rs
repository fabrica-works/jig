use super::{domain::spine::LIMITS, fixtures::Fixture};
use crate::objects::Source;
use jig_client::contract::Read;
#[test]
fn reads_from_one_window_keep_distinct_handles() {
    let mut fixture = Fixture::new(&LIMITS);
    let source = Source::Window { number: 1 };
    let first = fixture.request_read(source, Read::Task { task: 1 }).unwrap();
    let second = fixture.request_read(source, Read::Task { task: 2 }).unwrap();
    assert_ne!(first, second);
    assert_eq!(fixture.handles.reads.get(&first).unwrap().what, Read::Task { task: 1 });
    assert_eq!(fixture.handles.reads.get(&second).unwrap().what, Read::Task { task: 2 });
}
