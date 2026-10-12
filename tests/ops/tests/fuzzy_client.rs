//! Seeded random people in both Ops starting states, judged by the shared referee.
use jig_client_world::Until;
use jig_domain_client::address::Address as JigAddress;
use jig_ops_domain_client::{Address, observability as obs};
use jig_ops_world::client::stories;
use skein_world::domain::{Verdict, heap::Counting};
#[global_allocator]
static HEAP: Counting = Counting;
fn sweep(incident: bool) {
    let mut faults = 0;
    let mut judged = 0;
    for seed in 5050..5058 {
        eprintln!("Ops {} seed {seed}", if incident { "incident" } else { "environment chat" });
        let mut world = if incident { stories::incident_world(seed) } else { stories::environment_world(seed) };
        world.randomness(12, 150, 8);
        world.measure_heap();
        let address = if incident {
            Address::Observability(obs::Address::Incidents { project: 1, open: true })
        } else {
            Address::Jig(JigAddress::Conversations { project: None })
        };
        for party in [7, 8] {
            let client = world.client(party, address.clone());
            world.random_person(client);
            let tab = world.tab(client);
            world.random_person(tab);
        }
        let result = world.run(Until::Done);
        assert_eq!(result.verdict, Verdict::Passed, "incident={incident} seed={seed}");
        faults += result.faults;
        judged += result.judged.0;
        assert!(result.heap_peak <= result.heap_bound);
    }
    assert!(faults > 0, "fault coverage must be positive");
    assert!(judged > 0, "referee coverage must be positive");
}
#[test]
fn random_people_in_ops_incident_starts() {
    sweep(true);
}
#[test]
fn random_people_in_ops_environment_chat_starts() {
    sweep(false);
}
