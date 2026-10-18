//! Copy this shell with its host: platform facts enter, host duties leave.
#![cfg(target_arch = "wasm32")]
#![forbid(unsafe_code)]
use jig_client_shell_web::{adapter, look, sign_in};
use skein_lib::Queue;
use skein_ui::{Act, NodeId, Owner, look::Names};
use skein_ui_web::{
    Look, fetch, history,
    inbox::Inbox,
    page::{Page, web::Dom},
    platform,
    store::Store,
    timer, title, transport, wasm_bindgen,
};
use std::{cell::RefCell, rc::Rc};
mod host;
mod limits;
mod mapping;
mod records;
const OWNER: Owner = jig_client_view::Owner::Application { number: jig_client_test_domain::FIRST }.opaque();
const NAMES: &[Names<'_>] = &[look::NAMES, Names { owner: OWNER, prefix: b"jig-test", styles: &[], regions: &[] }];
enum Report {
    Start,
    Due,
    Act(NodeId, Act),
    Fetch(fetch::Report),
    Carried(Vec<transport::Carried>, bool),
    Place(Box<[u8]>),
}
struct Shell {
    host: host::Host,
    page: Rc<RefCell<Page<'static>>>,
    dom: Dom,
    platform: platform::Browser<Report>,
    store: Store,
    history: history::web::History,
    timer: timer::web::Timer,
    title: title::web::Title,
}
#[wasm_bindgen(start)]
pub fn start() {
    let words = look::phrases();
    if !skein_ui_web::restart::Restart::new(3, skein_lib::Duration::from_secs(60), "jig-restarts", words.failed)
        .install("app")
    {
        return;
    }
    // Replace the printed secret's address before starting anything that can send.
    let secret = sign_in::web::take_from_page(&sign_in::Route { path: b"/jig/v1/sign-in/link", home: b"/" });
    let window = web_sys::window().expect("window");
    let mount = window.document().expect("document").get_element_by_id("app").expect("bundle mount");
    let facts = platform::facts();
    let path = window.location().pathname().expect("path") + &window.location().search().expect("query");
    let inbox = Inbox::new();
    let platform = platform::Browser::new(fetch::web::Fetch::new(9, Rc::clone(&inbox), Report::Fetch));
    let store = Store::new(b"jig.", None);
    let mut host = host::Host::new(facts.seed, facts.now, facts.wall);
    #[cfg(debug_assertions)]
    if path
        .as_bytes()
        .strip_prefix(b"/test/prefixed")
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(b"/") || rest.starts_with(b"?"))
    {
        host.codec =
            jig_client_view::address::Codec { prefix: b"/test/prefixed", ..jig_client_view::address::REFERENCE };
    }
    host.start(path.as_bytes(), store.read_all(&platform), facts.offset, secret);
    let page = Rc::new(RefCell::new(Page::new(
        Look {
            names: NAMES,
            pictures: &[look::PICTURES, (OWNER, &[b"test.svg"])],
            draw: |_, _, _, _, _| false,
            components: &[],
            elements: &[],
        },
        &host.bounds.tree,
        b"jig",
        facts.primary,
    )));
    let dom = Dom::new(mount, &page, &inbox, Report::Act, &[]);
    let mut shell = Shell {
        host,
        page,
        dom,
        platform,
        store,
        history: history::web::History::new(Rc::clone(&inbox), Report::Place),
        timer: timer::web::Timer::new(Rc::clone(&inbox), || Report::Due),
        title: title::web::Title::default(),
    };
    inbox.start(Box::new(move |inbox| shell.turn(inbox)));
    inbox.raise(Report::Start);
}
impl Shell {
    fn turn(&mut self, inbox: &Inbox<Report>) {
        let (now, wall) = platform::clocks();
        self.host.clocks(now, wall);
        let mut new_page = false;
        while let Some(report) = inbox.next() {
            let read_on =
                matches!(&report, Report::Fetch(fetch::Report::StreamOpened | fetch::Report::StreamPiece { .. }));
            match report {
                Report::Start => {}
                Report::Due => {
                    self.timer.fired();
                    self.host.due();
                }
                Report::Act(node, act) => self.host.act(node, act),
                Report::Place(path) => self.host.went(&path),
                Report::Fetch(report) => self.host.heard(adapter::fetched(report)),
                Report::Carried(reports, stored) => self.host.carried(reports, stored),
            }
            let (asked, credentials) = self.host.requests();
            let requests = adapter::requests(asked);
            let stored = !requests.writes.is_empty() || !requests.removes.is_empty();
            let mut carried = Vec::new();
            transport::carry(requests, &mut self.store, &mut self.platform, &mut carried);
            if stored || !carried.is_empty() {
                inbox.raise(Report::Carried(carried, stored));
            }
            if read_on {
                self.platform.fetch.read_on();
            }
            if let Some((path, replace)) = self.host.show.take() {
                self.history.show(&path, replace);
                new_page = !replace;
            }
            for credential in credentials {
                sign_in::web::show_once(look::phrases(), credential.expose());
            }
        }
        let mut patches = Queue::with_capacity(self.host.bounds.tree.patches);
        self.host.build(&mut patches);
        let patches: Vec<_> = std::iter::from_fn(|| patches.pop()).collect();
        let mut ops = Vec::new();
        self.page.borrow_mut().apply(&patches, &mut ops);
        let links = self.page.borrow().links().to_vec();
        for node in links {
            if let Some(path) = self.host.href(node) {
                self.page.borrow_mut().set_href(node, &path, &mut ops);
            }
        }
        self.dom.apply(&ops);
        self.finish(&ops, new_page, now);
    }
    fn finish(&mut self, ops: &[skein_ui_web::page::Op], new_page: bool, now: skein_lib::Time) {
        let window = web_sys::window().expect("window");
        let document = window.document().expect("document");
        let heading = document.query_selector("main h1").expect("heading selector");
        let words = heading.as_ref().and_then(|element| element.text_content()).unwrap_or_default();
        // Session storage survives reload, but closing the tab loses its requests.
        let closing_loses = self.store.read_all(&self.platform).iter().any(|record| record.key.len() == 16);
        self.title.set(words.as_bytes(), b"jig test", b"en", closing_loses);
        if new_page {
            window.scroll_to_with_x_and_y(0.0, 0.0);
            if !ops.iter().any(|op| matches!(op, skein_ui_web::page::Op::Focus { .. })) {
                use wasm_bindgen::JsCast;
                if let Some(target) = heading.or_else(|| document.query_selector("main").expect("main selector")) {
                    target.set_attribute("tabindex", "-1").expect("focus target");
                    target.unchecked_ref::<web_sys::HtmlElement>().focus().expect("focus mounted target");
                }
            }
        }
        self.history.restore();
        self.timer.set(self.host.deadline(), now);
    }
}
