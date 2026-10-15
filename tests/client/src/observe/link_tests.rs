use super::{Seen, jig_seen, seen};
use skein_lib::Queue;
use skein_ui::Limits;
use skein_ui::{Builder, Role, Run, View};

fn status(builder: &mut Builder, name: &[u8]) {
    builder.open(Role::Status);
    builder.name(name);
    builder.text(Run::Plain, name);
    builder.close();
}
fn view(current: Option<&[u8]>, retained: &[u8]) -> View {
    let limits = Limits {
        nodes: 24,
        depth: 5,
        text_per_node: 64,
        text_bytes: 1024,
        drawing_bytes: 0,
        styles_per_node: 0,
        patches: 128,
        bindings: 0,
        slots: 0,
        markdown_depth: 1,
    };
    let mut view = View::new(&limits);
    let mut builder = view.builder();
    builder.open(Role::Page);
    // An unrelated status before the frame defeats a global first-status query.
    builder.open(Role::Status);
    builder.name(b"Notice");
    status(&mut builder, retained);
    builder.close();
    if let Some(current) = current {
        builder.open(Role::Region);
        builder.region(jig_view::regions::JigRegion::Link.region());
        status(&mut builder, current);
        builder.close();
        builder.open(Role::Navigation);
        builder.name(b"Navigation");
        status(&mut builder, retained);
        builder.close();
    }
    builder.open(Role::Status);
    builder.name(b"Notice");
    status(&mut builder, retained);
    builder.close();
    builder.close();
    view.finish(builder, 1, &mut Queue::with_capacity(limits.patches));
    view
}
#[test]
fn a_current_offline_frame_is_observed_independently_of_retained_live_notices() {
    let view = view(Some(b"Offline"), b"Live");
    assert_eq!(jig_seen(view.tree()), vec![Seen::Link { live: false }]);
    assert_eq!(seen(view.tree()), vec![Seen::Link { live: false }]);
}
#[test]
fn a_current_live_frame_is_observed_independently_of_retained_offline_notices() {
    let view = view(Some(b"Live"), b"Offline");
    assert_eq!(jig_seen(view.tree()), vec![Seen::Link { live: true }]);
    assert_eq!(seen(view.tree()), vec![Seen::Link { live: true }]);
}
#[test]
fn a_connecting_frame_does_not_inherit_link_observations_from_notices_or_navigation() {
    for retained in [b"Live".as_slice(), b"Offline"] {
        let view = view(Some(b"Connecting"), retained);
        assert!(jig_seen(view.tree()).is_empty());
        assert!(seen(view.tree()).is_empty());
    }
}
#[test]
fn a_notice_without_a_frame_does_not_claim_a_current_link() {
    for retained in [b"Live".as_slice(), b"Offline"] {
        let view = view(None, retained);
        assert!(jig_seen(view.tree()).is_empty());
        assert!(seen(view.tree()).is_empty());
    }
}
