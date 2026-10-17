use super::{
    objects,
    spine::{drive, open_token},
};
use crate::tests::fixtures::{ReadPage, Watched};
use crate::{
    Event, Request,
    address::{Address, Place, TaskSection},
    objects::Body,
};
use jig_client::contract::*;
use skein_lib::{List, Time};

#[test]
fn whole_owner_task_resources_survive_reads_without_invented_scope_while_reference_routes_stay_exact() {
    let mut segments = List::with_capacity(2);
    segments.push(Box::<[u8]>::from(*b"a/b")).unwrap();
    segments.push(Box::<[u8]>::from([255_u8, 0])).unwrap();
    let owner = OwnerResourceLine {
        part: OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Resource, key: OwnerPartKey::Path(segments) },
        kind: u16::MAX,
        taken: false,
        holder: Some(u64::MAX),
        writer: Some(OwnerResourceWriter::Effect { entry: u64::MAX }),
        writer_lost: true,
    };
    let mut path = List::with_capacity(1);
    path.push(Box::<[u8]>::from(*b"reference")).unwrap();
    let reference = Part::Resource { connector: 1, path };
    for include_reference in [false, true] {
        let (mut domain, page) = objects::live();
        let first = objects::watched(&mut domain, page, &[ObjectKey::Task(8)]);
        let read = objects::read_token(&first);
        let mut row = super::confirmation::task();
        row.resources = List::with_capacity(2);
        row.resources.push(TaskResource::Owner(Box::new(owner.clone()))).unwrap();
        if include_reference {
            row.resources.push(TaskResource::Part(reference.clone())).unwrap();
        }
        let exact = row.clone();
        let output = drive(
            &mut domain,
            Time::ZERO,
            ReadPage { read, result: ReadResult::Value(ReadValue::Task(Box::new(row))) },
        );
        let id = domain.objects().find(ObjectKey::Task(8)).unwrap();
        assert_eq!(domain.objects().get(id).unwrap().body, Some(Body::Task(Box::new(exact.clone()))));
        assert!(output.is_empty(), "object reads retain facts without selecting application parts");
        let opened = drive(
            &mut domain,
            Time::ZERO,
            Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Overview }) },
        );
        let watch = open_token(&opened);
        let output = drive(
            &mut domain,
            Time::ZERO,
            Watched { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(exact))) },
        );
        if include_reference {
            assert_eq!(output.as_ref(), &[Request::ShowPart { part: reference.clone(), scope: PartScope::Project(1) }]);
        } else {
            assert!(output.is_empty());
        }
        assert_eq!(TaskResource::Owner(Box::new(owner.clone())).part(), None);
        assert_eq!(TaskResource::Part(reference.clone()).part(), Some(&reference));
    }
}
