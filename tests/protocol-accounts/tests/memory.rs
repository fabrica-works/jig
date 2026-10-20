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
