//! The credential story's resource-bearing agent delegate.
use super::{World, connector, core, root, tasks};
use skein_lib::{ReplyTo, Token};

impl World {
    pub fn delegate_on_resource(&mut self, resource: u64) {
        self.configure_holds(
            1,
            Box::new([tasks::Kind {
                connector: 1,
                kind: 1,
                hold: tasks::HoldKind::Exclusive { taken: tasks::Taken::Waits },
            }]),
        );
        self.send(root::Event::Connector {
            number: 1,
            event: connector::Event::Adopt {
                project: 1,
                resource: jig_test_connector_world::path(1, u8::try_from(resource).expect("fixture resource")),
                role: connector::ResourceRole::Owned,
            },
        });
        let mut authority = self.assigned.as_ref().expect("assigned caller").run.authority.clone();
        authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
        authority.budget.spend = 20;
        self.send(root::Event::Core(core::Event::DelegateValidated {
            to: ReplyTo::new(Token::new(201)),
            key: self.call_key(1),
            batch: Box::new([core::Delegate {
                executor: tasks::Executor::Agent { charter: 1 },
                spec: tasks::Spec {
                    words: b"work on the resource".as_slice().into(),
                    parameters: Box::new([tasks::Parameter::Resource { name: 1, connector: 1, resource }]),
                    inputs: Box::new([]),
                },
                contract: tasks::Contract::Report { words: 128 },
                authority,
                symbolic_grants: Box::new([]),
                dependencies: Box::new([]),
                wake: tasks::WakePolicy::DEFAULT,
            }]),
            stubs: Box::new([]),
        }));
    }
}
