//! Compose the frame, owner's page, and two slot passes (root.md, section 9).
use crate::{
    Action, Limits,
    bindings::{self, Bindings},
    family, words,
};
use jig_client_view::{Act, Builder, JigBindings, NodeId, Owner, Patch, Role, Tree};
use jig_test_client_domain::{Connector, Domain, first};
use skein_lib::Queue;
#[derive(Debug)]
pub struct View {
    render: jig_client_view::View,
    words: jig_client_view::words::Words,
    first_words: words::Words,
    jig: JigBindings,
    first: Bindings,
    limits: Limits,
    navigation: first::Address,
}
impl View {
    #[must_use]
    pub fn new(
        limits: &Limits,
        domain: &jig_test_client_domain::Limits,
        words: jig_client_view::words::Words,
        first_words: words::Words,
        navigation: first::Address,
    ) -> Self {
        assert!(crate::worst_case(limits, domain, &words, &first_words).is_some(), "view capacities are checked");
        Self {
            render: jig_client_view::View::new(&limits.tree),
            jig: JigBindings::new(limits.tree.bindings, domain.jig.text),
            first: Bindings::new(limits.tree.bindings, limits.text),
            words,
            first_words,
            limits: *limits,
            navigation,
        }
    }
    #[must_use]
    pub fn tree(&self) -> &Tree {
        self.render.tree()
    }
    #[must_use]
    pub const fn limits(&self) -> &Limits {
        &self.limits
    }
}
pub fn build(view: &mut View, root: &Domain, out: &mut Queue<Patch>) {
    let shown = jig_test_client_domain::shown(root);
    if !view.render.stale(shown) {
        return;
    }
    view.jig.begin();
    view.first.begin();
    let mut b = view.render.builder();
    b.open(Role::Page);
    if !jig_client_view::jig(&mut b, root.jig(), family::NAV, &view.words, &mut view.jig) {
        family::page(&mut b, root.first(), &view.first_words, &view.words, &mut view.first);
        jig_client_view::after_page(&mut b, root.jig(), &view.words, &mut view.jig);
    }
    b.close();
    for pass in 0..2 {
        for slot in b.slots(pass) {
            let mut fragment = Builder::new(&view.limits.tree);
            let filled = match slot.reference.owner {
                Owner::Jig => match view.first.object(slot.reference.token) {
                    Some(key) => match root.jig().objects().find(key) {
                        Some(id) => jig_client_view::cards::card(
                            &mut fragment,
                            root.jig(),
                            id,
                            &jig_client_view::cards::Place::Elsewhere,
                            &view.words,
                            &mut view.jig,
                        ),
                        None => false,
                    },
                    None => false,
                },
                Owner::Side { connector } => match Connector::of(connector) {
                    Some(Connector::First) => {
                        if slot.reference == family::NAV {
                            family::navigation(&mut fragment, view.navigation, &view.first_words, &mut view.first);
                            true
                        } else {
                            match view.jig.part(slot.reference.token) {
                                Some(part) => {
                                    family::part(
                                        &mut fragment,
                                        root.first(),
                                        part,
                                        root.jig().generation(),
                                        &view.first_words,
                                        &view.words,
                                        &mut view.first,
                                    );
                                    true
                                }
                                None => false,
                            }
                        }
                    }
                    Some(Connector::Second) => match view.jig.part(slot.reference.token) {
                        Some(part) => {
                            second(&mut fragment, root, part, &view.first_words);
                            true
                        }
                        None => false,
                    },
                    None => false,
                },
                Owner::Other { .. } => false,
            };
            if filled {
                b.splice(slot.id, fragment);
            } else {
                b.unshowable(slot.id, &view.first_words.say(words::Word::Unshowable, &[]));
            }
        }
    }
    view.render.finish(b, shown, out);
}
fn second(b: &mut Builder, root: &Domain, part: &jig_client::contract::Part, words: &words::Words) {
    use jig_client::contract::Part;
    use jig_client_view::phrases::Hole;
    if !root.second().contains(part) {
        words.status(b, words::Word::Unshowable, &[]);
        return;
    }
    match part {
        Part::Connector { connector } => words.text(b, words::Word::Connector, &[Hole::Number(u64::from(*connector))]),
        Part::Resource { path, .. } => {
            words.text(b, words::Word::Resource, &[Hole::Name(skein_lib::bytes::copy_of(path.bytes()))]);
        }
        Part::Kind { kind, .. } => words.text(b, words::Word::Kind, &[Hole::Number(u64::from(*kind))]),
        Part::Panel { .. }
        | Part::Result { .. }
        | Part::Effect { .. }
        | Part::Outbox { .. }
        | Part::Hold { .. }
        | Part::Project { .. }
        | Part::Deployment { .. } => words.status(b, words::Word::Unshowable, &[]),
    }
}
#[must_use]
#[expect(clippy::manual_map, reason = "step code matches optional owner actions explicitly")]
pub fn decode(view: &mut View, node: NodeId, act: Act) -> Option<Action> {
    let bound = view.render.act(node, act)?;
    match bound.binding.owner {
        Owner::Jig => match jig_client_view::decode(&mut view.jig, bound.binding.token, bound.act) {
            Some(action) => Some(Action::Jig(action)),
            None => None,
        },
        Owner::Side { connector } => match Connector::of(connector) {
            Some(Connector::First) => match bindings::decode(&view.first, bound.binding.token, bound.act) {
                Some(action) => Some(Action::First(action)),
                None => None,
            },
            Some(Connector::Second) | None => None,
        },
        Owner::Other { .. } => None,
    }
}
