//! Keep: frame, owner page and two slot passes (root.md, 9).
//! Replace: connector fragments, family addresses and phrasebooks.
use crate::{
    Action, Limits,
    bindings::{self, Bindings},
    infrastructure, observability,
    words::{self, Word},
};
use jig_ops_client_domain::Domain;
use jig_ops_connectors::Connector;
use jig_view::{Act, Builder, JigBindings, NodeId, Owner, Patch, Role, Tree};
use skein_lib::Queue;
/// The composed semantic tree and its owner binding tables.
#[derive(Debug)]
pub struct View {
    render: jig_view::View,
    words: jig_view::words::Words,
    observability_words: words::Words,
    infrastructure_words: words::Words,
    jig: JigBindings,
    observability: Bindings,
    infrastructure: Bindings,
    limits: Limits,
    project: u32,
    native_environments: bool,
}
impl View {
    #[must_use]
    pub fn new(
        limits: &Limits,
        domain: &jig_ops_client_domain::Limits,
        words: jig_view::words::Words,
        observability_words: words::Words,
        infrastructure_words: words::Words,
        project: u32,
    ) -> Self {
        assert!(
            observability_words.connector == jig_ops_connectors::number(Connector::Observability)
                && infrastructure_words.connector == jig_ops_connectors::number(Connector::Infrastructure),
            "phrasebooks belong to their connector owners"
        );
        assert!(
            crate::worst_case(limits, domain, &words, &observability_words, &infrastructure_words).is_some(),
            "view capacities are checked"
        );
        Self {
            render: jig_view::View::new(&limits.tree),
            jig: JigBindings::new(limits.tree.bindings, domain.jig.text),
            observability: Bindings::new(
                limits.tree.bindings,
                domain.observability.text,
                jig_ops_connectors::number(Connector::Observability),
            ),
            infrastructure: Bindings::new(
                limits.tree.bindings,
                domain.infrastructure.text,
                jig_ops_connectors::number(Connector::Infrastructure),
            ),
            words,
            observability_words,
            infrastructure_words,
            limits: *limits,
            project,
            native_environments: false,
        }
    }
    /// Select the native owner environment vocabulary before building a tree.
    pub fn native_environments(&mut self) {
        self.native_environments = true;
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
#[expect(clippy::too_many_lines, reason = "the root exhaustively routes two connectors in both slot passes")]
pub fn build(view: &mut View, root: &Domain, out: &mut Queue<Patch>) {
    let shown = jig_ops_client_domain::shown(root);
    if !view.render.stale(shown) {
        return;
    }
    view.jig.begin();
    view.observability.begin();
    view.infrastructure.begin();
    let mut builder = view.render.builder();
    builder.open(Role::Page);
    if !jig_view::jig(&mut builder, root.jig(), observability::NAV, &view.words, &mut view.jig) {
        if root.observability().address().is_some() {
            observability::page(&mut builder, root.observability(), &view.observability_words, &mut view.observability);
        } else {
            infrastructure::page(
                &mut builder,
                root.infrastructure(),
                &view.infrastructure_words,
                &view.words,
                &mut view.infrastructure,
            );
        }
        jig_view::after_page(&mut builder, root.jig(), &view.words, &mut view.jig);
    }
    builder.close();
    for pass in 0..2 {
        for slot in builder.slots(pass) {
            let mut fragment = Builder::new(&view.limits.tree);
            let filled = match slot.reference.owner {
                Owner::Jig => {
                    let key = match view.observability.object(slot.reference.token) {
                        Some(key) => Some(key),
                        None => view.infrastructure.object(slot.reference.token),
                    };
                    match key {
                        Some(key) => match root.jig().objects().find(key) {
                            Some(id) => jig_view::cards::card(
                                &mut fragment,
                                root.jig(),
                                id,
                                &jig_view::cards::Place::Elsewhere,
                                &view.words,
                                &mut view.jig,
                            ),
                            None => false,
                        },
                        None => false,
                    }
                }
                Owner::Side { connector } => match jig_ops_connectors::connector(connector) {
                    Some(Connector::Observability) => {
                        if slot.reference == observability::NAV {
                            observability::navigation(
                                &mut fragment,
                                view.project,
                                &view.observability_words,
                                &mut view.observability,
                            );
                            infrastructure::navigation(
                                &mut fragment,
                                view.project,
                                view.native_environments,
                                &view.infrastructure_words,
                                &mut view.infrastructure,
                            );
                            true
                        } else {
                            match view.jig.part(slot.reference.token) {
                                Some(part) => {
                                    observability::part(
                                        &mut fragment,
                                        root.observability(),
                                        part,
                                        root.jig().generation(),
                                        &view.observability_words,
                                        &mut view.observability,
                                    );
                                    true
                                }
                                None => false,
                            }
                        }
                    }
                    Some(Connector::Infrastructure) => match infrastructure_part(&view.jig, slot.reference.token) {
                        Some(part) => {
                            infrastructure::part(
                                &mut fragment,
                                root.infrastructure(),
                                &part,
                                root.jig().generation(),
                                &view.infrastructure_words,
                                &view.words,
                                &mut view.infrastructure,
                            );
                            true
                        }
                        None => false,
                    },
                    None => false,
                },
                Owner::Other { .. } => false,
            };
            if filled {
                builder.splice(slot.id, fragment);
            } else {
                builder.unshowable(slot.id, &view.observability_words.say(Word::Unshowable, &[]));
            }
        }
    }
    view.render.finish(builder, shown, out);
}
fn infrastructure_part(bindings: &JigBindings, token: skein_lib::Token) -> Option<jig_domain_client::contract::Part> {
    match bindings.part(token) {
        Some(part) => Some(part.clone()),
        None => match bindings.owner_part(token) {
            Some(part) => jig_domain_client::application::owner_effect(part),
            None => None,
        },
    }
}
#[must_use]
#[expect(clippy::manual_map, reason = "step code explicitly wraps the owner action")]
pub fn decode(view: &mut View, node: NodeId, act: Act) -> Option<Action> {
    let bound = view.render.act(node, act)?;
    match bound.binding.owner {
        Owner::Jig => match jig_view::decode(&mut view.jig, bound.binding.token, bound.act) {
            Some(value) => Some(Action::Jig(value)),
            None => None,
        },
        Owner::Side { connector } => match jig_ops_connectors::connector(connector) {
            Some(Connector::Observability) => bindings::decode(&view.observability, bound.binding.token, bound.act),
            Some(Connector::Infrastructure) => bindings::decode(&view.infrastructure, bound.binding.token, bound.act),
            None => None,
        },
        Owner::Other { .. } => None,
    }
}
