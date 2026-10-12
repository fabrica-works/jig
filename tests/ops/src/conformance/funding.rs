//! A chat starts under its smaller role; its owner later grants the
//! authority needed to accept the proposed work.
use super::{Input, Peers, Store, config, core, people, root};
use skein_lib::{ReplyTo, Token};

pub(super) fn poll(peers: &mut Peers, store: &Store) -> Vec<Input<root::Event, root::Record>> {
    let sign_in = peers.sign_in.or_else(|| {
        store.rows.values().find_map(|row| match row {
            root::Record::Core(core::Record::People(people::Stored::SignIn { number, person: 1, .. })) => Some(*number),
            root::Record::Core(_) | root::Record::Infrastructure(_) | root::Record::Observability(_) => None,
        })
    });
    let Some(sign_in) = sign_in else { return Vec::new() };
    if peers.role_widened {
        return Vec::new();
    }
    let Some(environment) = store.rows.values().find_map(|row| match row {
        root::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::Live(task)))
            if task.spec.words.as_ref() == b"@watch_setup" =>
        {
            Some(false)
        }
        root::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::Live(task)))
            if task.spec.words.starts_with(b"@environment_chat") =>
        {
            Some(true)
        }
        root::Record::Core(_) | root::Record::Infrastructure(_) | root::Record::Observability(_) => None,
    }) else {
        return Vec::new();
    };
    peers.role_widened = true;
    let value = if environment { config::environment_authority(500, true) } else { config::authority(500, true) };
    let kinds = value
        .delegation
        .kinds
        .iter()
        .map(|kind| match kind {
            jig_domain_authority::Executor::Charter(number) => people::Executor::Charter(*number),
            jig_domain_authority::Executor::Procedure(number) => people::Executor::Procedure(*number),
            jig_domain_authority::Executor::Role(number) => people::Executor::Role(*number),
        })
        .collect();
    let grants = value
        .grants
        .into_iter()
        .map(|grant| people::Grant {
            connector: grant.connector,
            kind: grant.kind,
            pattern: people::Pattern {
                segments: grant.pattern.segments,
                last: match grant.pattern.last {
                    jig_domain_authority::Last::Exact(bytes) => people::Last::Exact(bytes),
                    jig_domain_authority::Last::Open(bytes) => people::Last::Open(bytes),
                },
            },
        })
        .collect();
    vec![Input::Event(root::Event::Core(core::Event::People(people::Event::Ask {
        reply_to: ReplyTo::new(Token::new(90020)),
        sign_in,
        key: [78; 16],
        ask: people::Ask::ChangePolicy {
            project: 1,
            change: people::PolicyChange::Role(people::PolicyRole {
                number: 0,
                authority: people::Authority {
                    tools: value.tools.0,
                    grants,
                    delegation: people::Delegation {
                        kinds,
                        tasks: value.delegation.tasks,
                        depth: value.delegation.depth,
                    },
                    spend: value.budget.spend,
                    deadline: None,
                    notes: 0,
                    note_resources: Box::new([]),
                },
                period_spend: 500,
                requests: jig_domain_authority::Requests::ALL.0,
                decides: jig_domain_authority::Proposals::ALL.0,
            }),
        },
    })))]
}
