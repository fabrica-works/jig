use super::fixtures::{confirm, drive, limits, start};
use crate::{Action, Domain, Event};
use jig_client::contract::*;
use jig_client_view::{JigBindings, Owner};
use skein_lib::{List, Token};
use skein_ui::{Act, Builder, Role, SlotRef, Tree};

const PERSON: u64 = 7;
const PROJECT: u32 = 1;
const CHARTER: u32 = 1;
const TEXT: u32 = 32;

#[test]
fn the_request_stays_in_the_loaded_client() {
    let mut root = crate::unlinked(&limits(), PERSON);
    let watch = start(&mut root, List::with_capacity(0));
    confirm(&mut root, watch, PERSON);
    let mut frame = root.jig().borrow(root.client()).frame().document.unwrap().clone();
    frame.projects = chat_project();
    drive(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: WatchEvent::Delivered(Delivery::Me(Box::new(frame))) },
            },
        },
    );
    let (tree, mut bindings) = render(&root);
    let link = tree
        .nodes()
        .iter()
        .find(|node| node.role == Role::Link && node.name.as_deref() == Some(b"Conversations"))
        .expect("inbox offers Conversations");
    let action = jig_client_view::decode(&mut bindings, link.binding.unwrap().token, Act::Press).unwrap();
    drive(&mut root, Event::Act { action: Action::Jig(action) });
    let (tree, _) = render(&root);
    for (role, name) in
        [(Role::TextArea, b"First words".as_slice()), (Role::Button, b"Start a conversation".as_slice())]
    {
        assert!(
            tree.nodes().iter().any(|node| node.role == role && node.name.as_deref() == Some(name)),
            "missing {role:?} {} without a server document",
            String::from_utf8_lossy(name)
        );
    }
}

fn chat_project() -> List<MeProject> {
    let mut templates = List::with_capacity(1);
    templates
        .push(Template {
            llm: None,
            number: jig_client::shapes::CHAT,
            name: DocumentText::complete(Text::new(b"Chat".as_slice().into(), TEXT).unwrap()),
            executor: TemplateExecutor::Charter(CHARTER),
            contract: TaskContractDocument::Report { words: TEXT },
            shallower: 1,
            budget: TemplateBudget::AsAllowed { cap: None },
            deadline: TemplateDeadline::None,
            tracked: None,
            flags: TaskFlags { conversation: true, listed: false, retention: Retention::Kept },
            label: 1,
            grace: None,
        })
        .unwrap();
    let mut projects = List::with_capacity(1);
    projects
        .push(MeProject {
            models: List::with_capacity(0),
            project: PROJECT,
            name: None,
            role: HeldRole::Policy { role: 1 },
            templates,
            creating: None,
            pool: None,
        })
        .unwrap();
    projects
}

fn render(root: &Domain) -> (Tree, JigBindings) {
    let bounds = skein_ui::Limits {
        nodes: 512,
        depth: 32,
        text_per_node: 1024,
        text_bytes: 16384,
        drawing_bytes: 0,
        styles_per_node: 2,
        patches: 1536,
        bindings: 64,
        slots: 8,
        markdown_depth: 3,
        scene_marks: 0,
        scene_bytes: 0,
    };
    let words = jig_client_view::words::Words::new(jig_client_view::phrases::english::english(TEXT).unwrap(), 0, 0);
    let mut bindings = JigBindings::new(bounds.bindings, TEXT);
    let mut builder = Builder::new(&bounds);
    builder.open(Role::Page);
    jig_client_view::jig(
        &mut builder,
        &root.jig().borrow(root.client()),
        SlotRef { owner: (Owner::Application { number: 1 }).opaque(), token: Token::new(1) },
        &words,
        &mut bindings,
    );
    builder.close();
    (builder.finish(), bindings)
}
