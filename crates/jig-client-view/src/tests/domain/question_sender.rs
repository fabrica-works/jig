use super::{
    objects::{live, question, text},
    spine::drive,
};
use crate::{
    Event, Request,
    action::Act,
    fields::Purpose,
    objects::{self, Body, Source},
};
use jig_client::contract::*;
use skein_lib::{List, Time, Wall};

#[test]
fn question_sender_closed_widths_and_absence_keep_whole_facts_and_original_answer_identity() {
    for from in [
        None,
        Some(OwnerRequester::Task(0)),
        Some(OwnerRequester::Task(u64::MAX)),
        Some(OwnerRequester::Person(0)),
        Some(OwnerRequester::Person(u64::MAX)),
        Some(OwnerRequester::Deployment { project: 0 }),
        Some(OwnerRequester::Deployment { project: u32::MAX }),
    ] {
        let (mut domain, _) = live();
        let mut supplied = question();
        supplied.from = from;
        supplied.message = u64::MAX;
        supplied.words = text(UTF8_TEXT);
        supplied.since = Wall::from_nanos(u64::MAX);
        supplied.acts = List::with_capacity(2);
        supplied.acts.push(Offered { act: ActKind::Answer, admission: Admission::Allowed }).unwrap();
        let key = ObjectKey::Question { task: supplied.task.task, message: supplied.message };
        let id = domain.objects.name(key, Source::Window { number: 4 }).unwrap();
        assert!(domain.objects.put(key, Body::Question(Box::new(supplied.clone()))));
        domain.objects.get_mut(id).unwrap().project = Some(1);
        assert_eq!(objects::question::facts(&domain.objects().get(id).unwrap()), Some(&supplied));
        drive(&mut domain, Time::ZERO, Event::Act { action: Act::Intend { object: id.token(), act: ActKind::Answer } });
        drive(
            &mut domain,
            Time::ZERO,
            Event::Act { action: Act::Edit { field: Purpose::Answer, text: text(b"whole answer") } },
        );
        let requests = drive(&mut domain, Time::ZERO, Event::Act { action: Act::Confirm });
        let mut sent = None;
        for request in requests.as_ref() {
            if let Request::Client { asked: jig_client::Event::Send { request: jig_client::Ask::Jig(ask), .. } } =
                request
            {
                sent = Some(ask);
                break;
            }
        }
        let sent = sent.expect("original question Answer request");
        assert_eq!(
            sent.as_ref(),
            &Ask::AnswerQuestion {
                project: 1,
                task: supplied.task.task,
                message: u64::MAX,
                words: text(b"whole answer")
            }
        );
        assert_eq!(objects::question::facts(&domain.objects().get(id).unwrap()), Some(&supplied));
    }
}

const UTF8_TEXT: &[u8] = b"~\0!?";
