//! Keep: owner-written bounded data and readable fallback (skein's ui.md, 3.5).
//! Replace: observability's error-rate drawing.
use crate::words::{Word, Words};
use jig_client_view::Owner;
use jig_ops_client_domain::observability::documents::Incident;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, DrawingKind, Run};

pub const BYTES: u32 = 32;
pub fn rate(b: &mut Builder, words: &Words, incident: &Incident) {
    let Some(health) = incident.health else { return };
    open_rate(b, words, health.error_rate_percent, health.observed);
    for point in &incident.timeline {
        if let Some(before) = point.health
            && before.observed < health.observed
        {
            b.text(Run::Plain, b", ");
            words.text(b, Word::Before, &[Hole::Number(u64::from(before.error_rate_percent))]);
            b.text(Run::Plain, b" %");
            break;
        }
    }
    b.close();
}

fn open_rate(b: &mut Builder, words: &Words, percent: u8, observed: skein_lib::Wall) {
    let mut data = skein_lib::Writer::new(usize::try_from(BYTES).expect("drawing bound fits"));
    data.put(&[percent]).expect("fixed drawing capacity");
    data.put(&observed.as_nanos().to_le_bytes()).expect("fixed drawing capacity");
    data.put(&[0; 23]).expect("fixed drawing capacity");
    let data = data.finish();
    b.drawing(
        DrawingKind { owner: (Owner::Application { number: words.connector }).opaque(), kind: 1 },
        &words.say(Word::ErrorRate, &[]),
        &data,
        BYTES,
    );
    words.text(b, Word::Rate, &[Hole::Number(u64::from(percent))]);
    b.text(Run::Plain, b" % ");
    words.text(b, Word::Since, &[Hole::Name(skein_lib::bytes::copy_of(&crate::time::clock(observed)))]);
}

pub fn native_rate(
    b: &mut Builder,
    words: &Words,
    incident: &jig_ops_client_domain::observability::native::Incident,
    fact: &jig_ops_client_domain::observability::native::Fact,
) {
    let observed = skein_lib::Wall::from_nanos(fact.observed.saturating_mul(1_000_000_000));
    open_rate(b, words, fact.error_rate_percent, observed);
    for point in &incident.timeline {
        if let Some(before) = point.health
            && before.observed < fact.observed
        {
            b.text(Run::Plain, b", ");
            words.text(b, Word::Before, &[Hole::Number(u64::from(before.error_rate_percent))]);
            b.text(Run::Plain, b" %");
            break;
        }
    }
    b.close();
}
