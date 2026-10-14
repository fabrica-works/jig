//! Compose the frame, owner's page, and two slot passes (root.md, section 9).
use crate::{
    Action, Limits,
    bindings::{self, Bindings},
    family, words,
};
use jig_test_domain_client::{Connector, Domain, first};
use jig_view::{Act, Builder, JigBindings, NodeId, Owner, Patch, Role, Tree};
use skein_lib::Queue;
#[derive(Debug)]
pub struct View<'a> {
    render: jig_view::View,
    words: &'a jig_view::words::Words,
    first_words: &'a words::Words,
    jig: JigBindings,
    pub(crate) first: Bindings,
    limits: Limits,
    navigation: first::Address,
}
impl<'a> View<'a> {
    #[must_use]
    pub fn new(
        limits: &Limits,
        domain: &jig_test_domain_client::Limits,
        words: &'a jig_view::words::Words,
        first_words: &'a words::Words,
        navigation: first::Address,
    ) -> Self {
        assert!(crate::worst_case(limits, domain, words, first_words).is_some(), "view capacities are checked");
        Self {
            render: jig_view::View::new(&limits.tree),
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
pub fn build(view: &mut View<'_>, root: &Domain, out: &mut Queue<Patch>) {
    let shown = jig_test_domain_client::shown(root);
    if !view.render.stale(shown) {
        return;
    }
    view.jig.begin();
    view.first.begin();
    let mut b = view.render.builder();
    b.open(Role::Page);
    if !jig_view::jig(&mut b, root.jig(), family::NAV, view.words, &mut view.jig) {
        b.open(Role::Main);
        family::page(&mut b, root.first(), view.first_words, view.words, &mut view.first);
        b.close();
        jig_view::after_page(&mut b, root.jig(), view.words, &mut view.jig);
    }
    b.close();
    slots(view, root, &mut b);
    view.render.finish(b, shown, out);
}
pub(crate) fn slots(view: &mut View<'_>, root: &Domain, b: &mut Builder) {
    for pass in 0..2 {
        for slot in b.slots(pass) {
            let mut fragment = Builder::new(&view.limits.tree);
            let filled = match slot.reference.owner {
                Owner::Jig => match view.first.object(slot.reference.token) {
                    Some(key) => match root.jig().objects().find(key) {
                        Some(id) => jig_view::cards::card(
                            &mut fragment,
                            root.jig(),
                            id,
                            &jig_view::cards::Place::Elsewhere,
                            view.words,
                            &mut view.jig,
                        ),
                        None => false,
                    },
                    None => match view.first.section(slot.reference.token) {
                        Some(section) => {
                            jig_view::sections::section(&mut fragment, root.jig(), view.words, &mut view.jig, section)
                        }
                        None => false,
                    },
                },
                Owner::Application { number } => match Connector::of(number) {
                    Some(Connector::First) => {
                        if slot.reference == family::NAV {
                            family::navigation(&mut fragment, view.navigation, view.first_words, &mut view.first);
                            true
                        } else {
                            match view.jig.part(slot.reference.token) {
                                Some(part) => {
                                    family::part(
                                        &mut fragment,
                                        root.first(),
                                        part,
                                        root.jig().generation(),
                                        view.first_words,
                                        view.words,
                                        &mut view.first,
                                    );
                                    true
                                }
                                None => match view.jig.owner_part(slot.reference.token) {
                                    Some(owner) => {
                                        let part = match jig_test_domain_client::family::owner_hold(owner) {
                                            Some(part) => Some(part),
                                            None => jig_test_domain_client::family::owner_effect(owner),
                                        };
                                        match part {
                                            Some(part) => {
                                                family::part(
                                                    &mut fragment,
                                                    root.first(),
                                                    &part,
                                                    root.jig().generation(),
                                                    view.first_words,
                                                    view.words,
                                                    &mut view.first,
                                                );
                                                true
                                            }
                                            None => false,
                                        }
                                    }
                                    None => false,
                                },
                            }
                        }
                    }
                    Some(Connector::Second) => match view.jig.part(slot.reference.token) {
                        Some(part) => {
                            second(&mut fragment, root, part, view.first_words, view.words);
                            true
                        }
                        None => false,
                    },
                    None => false,
                },
            };
            if filled {
                b.splice(slot.id, fragment);
            } else {
                b.unshowable(slot.id, &view.first_words.say(words::Word::Unshowable, &[]));
            }
        }
    }
}
fn second(
    b: &mut Builder,
    root: &Domain,
    part: &jig_domain_client::contract::Part,
    words: &words::Words,
    jig: &jig_view::words::Words,
) {
    use jig_domain_client::contract::Part;
    use jig_view::phrases::Hole;
    if !root.second().contains(part) {
        words.status(b, words::Word::Unshowable, &[]);
        return;
    }
    match part {
        Part::Connector { connector } => words.text(b, words::Word::Connector, &[Hole::Number(u64::from(*connector))]),
        Part::Resource { path, .. } => {
            jig.text(b, jig_view::phrases::jig::JigPhrase::ResourceSegments, &[]);
            for segment in path {
                jig_view::literal_segment::render(b, segment);
            }
        }
        Part::Kind { kind, .. } => words.text(b, words::Word::Kind, &[Hole::Number(u64::from(*kind))]),
        Part::Panel { .. }
        | Part::Result { .. }
        | Part::Effect { .. }
        | Part::OwnerEffect { .. }
        | Part::Outbox { .. }
        | Part::Hold { .. }
        | Part::Project { .. }
        | Part::Deployment { .. } => words.status(b, words::Word::Unshowable, &[]),
    }
}
#[must_use]
#[expect(clippy::manual_map, reason = "step code matches optional owner actions explicitly")]
pub fn decode(view: &mut View<'_>, node: NodeId, act: Act) -> Option<Action> {
    let bound = view.render.act(node, act)?;
    match bound.binding.owner {
        Owner::Jig => match jig_view::decode(&mut view.jig, bound.binding.token, bound.act) {
            Some(action) => Some(Action::Jig(action)),
            None => None,
        },
        Owner::Application { number } => match Connector::of(number) {
            Some(Connector::First) => match bindings::decode(&view.first, bound.binding.token, bound.act) {
                Some(action) => Some(Action::First(action)),
                None => None,
            },
            Some(Connector::Second) | None => None,
        },
    }
}
