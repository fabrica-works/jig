use super::cards::{contains, document, drive, start, text, tree};
use crate::tests::fixtures::Watched;
use crate::{
    Event,
    action::Act,
    address::{Address, Place},
    fields::Purpose,
};
use jig_client::contract::{Delivery, Refusal, WatchEvent};
const PROJECT: u32 = 4;
#[test]
fn a_chat_without_creating_sends_nothing_and_shows_the_refusal() {
    let (mut fixture, watch) = start();
    let mut me = document();
    me.projects.get_mut(0).unwrap().creating = None;
    drive(&mut fixture, Watched { watch, event: WatchEvent::Delivered(Delivery::Me(Box::new(me))) });
    drive(&mut fixture, Event::Went { place: Place::Jig(Address::Conversations { project: Some(PROJECT) }) });
    drive(&mut fixture, Event::Act { action: Act::Edit { field: Purpose::NewChat, text: text(b"not offered") } });
    assert!(drive(&mut fixture, Event::Act { action: Act::StartChat { project: PROJECT } }).is_empty());
    assert_eq!(fixture.chats.refusal, Some(Refusal::NotOffered));
    assert!(contains(&tree(&fixture, None, None), b"This act is not offered"));
}
