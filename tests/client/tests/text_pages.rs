use jig_client_world::{ClientId, Fault, FrontFault, Scripted, Test, Until, World, stories};
use jig_domain_client::address::{Address, ProjectSection};
use jig_fake_person::{Find, Step};
use jig_test_domain_client::{Address as RootAddress, Event, Place};
use jig_view::Act;
use jig_view::ui::Role;
use skein_lib::{Duration, Time, Wall};
type PageWorld = World<Test, Scripted>;
fn print(w: &PageWorld, c: ClientId, label: &str, golden: &[u8]) {
    let (_, view) = jig_client_world::test::new(&jig_client_world::test::limits(), 0);
    let mut print = jig_shell_text::Print::new(&view.limits().tree).expect("print fits");
    jig_shell_text::render(w.tree(c), b"jig", &mut print);
    assert_eq!(print.text.as_slice(), golden, "{label}");
}
fn settle(w: &mut PageWorld, c: ClientId) {
    w.person(c, vec![Step::Pause { span: Duration::from_millis(100) }]);
    w.run(Until::Done);
}
fn ready(mut w: PageWorld, address: RootAddress, label: &str, golden: &[u8]) {
    let c = if label == "inbox" {
        w.client_with(7, address, stories::text_pages::limits(), stories::text_pages::client)
    } else {
        w.client(7, address)
    };
    settle(&mut w, c);
    print(&w, c, label, golden);
}
#[test]
fn the_inbox_prints_as_its_golden_text() {
    ready(stories::text_pages::inbox(), stories::inbox(), "inbox", include_bytes!("golden/inbox.txt"));
}
#[test]
fn a_task_page_with_its_conversation_prints_as_its_golden_text() {
    ready(stories::proposal_world(92), stories::chat(), "task", include_bytes!("golden/task.txt"));
}
#[test]
fn a_projects_sections_print_as_their_golden_text() {
    ready(
        stories::goal_world(93),
        RootAddress::Jig(Address::Project { project: 1, section: ProjectSection::Settings }),
        "project",
        include_bytes!("golden/project.txt"),
    );
}
#[test]
fn the_deployment_prints_as_its_golden_text() {
    let mut w = stories::world(94);
    w.front()
        .engine
        .act(Time::ZERO, Wall::EPOCH, jig_scripted_engine::Change::Deployment(stories::deployment()))
        .expect("deployment fits");
    ready(w, RootAddress::Jig(Address::Deployment), "deployment", include_bytes!("golden/deployment.txt"));
}
#[test]
fn each_page_not_about_the_work_prints_as_its_golden_text() {
    for (page, golden) in [
        ("starting", include_bytes!("golden/starting.txt").as_slice()),
        ("signing_in", include_bytes!("golden/signing_in.txt")),
        ("missing", include_bytes!("golden/missing.txt")),
        ("out_of_date", include_bytes!("golden/out_of_date.txt")),
        ("someone_else", include_bytes!("golden/someone_else.txt")),
    ] {
        let (w, c) = special(page);
        print(&w, c, page, golden);
    }
}

fn special(page: &str) -> (PageWorld, ClientId) {
    let mut w = stories::chat_world(95);
    w.run(Until::Done);
    if page == "signing_in" {
        w.front()
            .engine
            .act(Time::ZERO, Wall::EPOCH, jig_scripted_engine::Change::SignedOut { sign_in: 7 })
            .expect("sign out");
    }
    let c = w.client(7, stories::conversation());
    if page != "starting" {
        settle(&mut w, c);
    }
    match page {
        "missing" => {
            w.event(c, Event::Went { place: Place::Missing });
            w.run(Until::Done);
        }
        "out_of_date" => {
            w.fault(FrontFault::Versions);
            settle(&mut w, c);
        }
        "someone_else" => {
            w.person(
                c,
                vec![
                    Step::Type { find: Find::named(Role::TextArea, b"First words"), words: Box::from(*b"Hello") },
                    Step::Pause { span: Duration::from_millis(20) },
                ],
            );
            w.run(Until::Done);
            let node = w
                .tree(c)
                .nodes()
                .iter()
                .find(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Start a conversation"))
                .expect("start button")
                .id;
            w.act(c, node, Act::Press);
            w.fault(Fault::OtherParty { client: c, party: 8 });
            settle(&mut w, c);
        }
        "starting" | "signing_in" => {}
        _ => unreachable!("named page"),
    }
    (w, c)
}
