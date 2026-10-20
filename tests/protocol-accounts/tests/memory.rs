use jig_protocol_accounts::{
    ACCOUNT_ID_BYTES,
    table::{self, Scheme, Table, Value},
};
use skein_heap::{Counting, Meter};
use skein_lib::{Secret, Time, bytes};
#[global_allocator]
static HEAP: Counting = Counting;
#[test]
fn the_table_at_its_limits_stays_within_its_worst_case() {
    let accounts = 2;
    let token_bytes = 32;
    let bound = table::worst_case(accounts, token_bytes).expect("table bound");
    let meter = Meter::new();
    meter.start();
    let mut table = Table::new(accounts, token_bytes);
    let token_length = usize::try_from(token_bytes).expect("token bound");
    let id_length = usize::try_from(ACCOUNT_ID_BYTES).expect("id bound");
    for account in 0..accounts {
        for generation in 1..=2 {
            table
                .granted(
                    account,
                    generation,
                    Value {
                        token: Secret::new(bytes::zeroed(token_length)),
                        scheme: Scheme::Withdrawn { id: Secret::new(bytes::zeroed(id_length)) },
                    },
                    Time::from_nanos(1000),
                )
                .expect("full value fits");
        }
    }
    let measured = meter.end();
    meter.check(measured, bound, &"accounts table");
}

#[test]
fn llm_keys_being_adopted_stay_within_their_worst_case() {
    use jig_protocol_accounts::keys::{self, Keys, Next};
    let bound = keys::worst_case(2, 32).expect("LLM key state bound");
    let meter = Meter::new();
    meter.start();
    let mut keys = Keys::new(2, 32);
    for source in 1..=2 {
        let Next::Store { exchange, .. } = keys.keep(source, u64::from(source), false) else { panic!("carried read") };
        let next = keys.read(exchange, Some(Secret::new(bytes::zeroed(32))));
        assert!(matches!(next, Some(Next::Store { .. })));
    }
    let measured = meter.end();
    meter.check(measured, bound, &"LLM key state");
}

#[test]
fn the_source_inventory_stays_within_its_worst_case() {
    use jig_protocol_accounts::restart::{self, Inventory};
    let bound = restart::worst_case(2, 2).expect("source inventory bound");
    let meter = Meter::new();
    meter.start();
    let mut inventory = Inventory::new(2, 2);
    inventory.name(1);
    inventory.name(2);
    inventory.row(3);
    inventory.row(4);
    let measured = meter.end();
    meter.check(measured, bound, &"source inventory");
}
