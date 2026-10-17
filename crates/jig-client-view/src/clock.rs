//! Wall time is for the view; deadlines are monotonic. A minute's tick is
//! armed only while a work page shows wall times; client/domain/client.md, 13.
use crate::{Ui, objects::Body, page::Page, ui::Timer};
use skein_lib::Duration;
/// Whether page policy shows times, and whether its one tick is armed.
#[derive(Debug)]
pub(crate) struct State {
    pub page_times: bool,
    pub armed: bool,
}
fn showing(domain: &Ui) -> bool {
    match domain.page {
        Page::Work { .. } => {}
        Page::Starting | Page::Elsewhere | Page::SigningIn | Page::Missing | Page::OutOfDate | Page::SomeoneElse => {
            return false;
        }
    }
    if domain.clock.page_times
        || !domain.turn_rows().rows.is_empty()
        || !domain.history_rows.rows.is_empty()
        || !domain.conversation_rows.rows.is_empty()
    {
        return true;
    }
    for (_, id) in &domain.objects.index {
        let object = domain.objects.slab.get(*id).expect("indexed object exists");
        if !object.sources.is_empty() {
            match &object.body {
                Some(
                    Body::Task(_)
                    | Body::Proposal(_)
                    | Body::OwnerProposal(_)
                    | Body::Escalation(_)
                    | Body::OwnerEscalation(_)
                    | Body::Question(_)
                    | Body::PersonTask(_)
                    | Body::OwnerPersonTask(_)
                    | Body::Result(_)
                    | Body::Note { .. },
                ) => return true,
                Some(Body::Historical(_) | Body::Chip(_)) | None => {}
            }
        }
    }
    false
}
pub(crate) fn sync(domain: &mut Ui) {
    if showing(domain) {
        if !domain.clock.armed {
            domain
                .timers
                .arm(Timer::Minute, domain.now.saturating_add(Duration::from_secs(60)))
                .expect("one minute timer fits");
            domain.clock.armed = true;
        }
    } else {
        domain.timers.cancel(Timer::Minute);
        domain.clock.armed = false;
    }
}
pub(crate) fn minute(domain: &mut Ui) {
    domain.clock.armed = false;
    if showing(domain) {
        domain.changed();
    }
    sync(domain);
}
