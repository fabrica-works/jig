use super::*;
use crate::address::{Address, TaskSection};

fn budget_page(row: Task) -> Tree {
    let (mut domain, out) = super::super::work_pages::entered(Address::Task { task: 8, section: TaskSection::Budget });
    let watch =
        out.iter()
            .find_map(|request| match request {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Task { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
    super::super::pages::deliver(&mut domain, watch, Delivery::Task(Box::new(row)));
    let tree = super::super::pages::build(&domain, 3, words()).0;
    super::super::cards_access::check(&tree);
    assert!(tree.len() <= crate::task_nodes(&CLIENT, 0).unwrap());
    tree
}

#[test]
fn task_budget_page_renders_all_owner_funders_numbers_and_deadline_presence_within_the_whole_bound() {
    for (funder, caption) in [
        (Funder::Task(u64::MAX), b"Funded by task ".as_slice()),
        (Funder::Pool { project: u32::MAX, person: u64::MAX, period: u64::MAX }, b"Funded by pool in project "),
        (
            Funder::Recurring { project: u32::MAX, task: u64::MAX, period: u64::MAX },
            b"Funded by recurring task in project ",
        ),
        (Funder::Period { project: u32::MAX, period: u64::MAX }, b"Funded by project period "),
    ] {
        for deadline in [None, Some(Wall::EPOCH), Some(Wall::from_nanos(u64::MAX))] {
            let mut row = task_doc(&[]);
            row.budget = OwnerBudget {
                numbers: OwnerNumbers { budget: u64::MAX, spent: 4, spent_below: 7, reserved: 11 },
                left: 17,
                funder,
                deadline,
            };
            row.authority.authority.deadline = Some(Wall::from_nanos(123));
            let tree = budget_page(row);
            for number in [
                b"Budget: 18446744073709551615".as_slice(),
                b"Spent directly: 4",
                b"Settled spend below: 7",
                b"Reserved for funded work: 11",
                b"Left: 17",
            ] {
                assert!(contains(&tree, number));
            }
            assert!(contains(&tree, caption));
            assert_eq!(contains(&tree, b"Deadline:"), deadline.is_some());
            assert!(!contains(&tree, b"Left: 78"));
        }
    }
}
