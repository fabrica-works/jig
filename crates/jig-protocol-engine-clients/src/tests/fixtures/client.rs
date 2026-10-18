use super::*;

pub fn client_limits() -> jig_client_protocol_link::Limits {
    let client = jig_client_domain::Limits {
        owners: &[],
        watches: 4,
        handles: 8,
        turn_bytes: 32,
        ask_bytes: 16384,
        requests: skein_lib::RequestLimits {
            requests: 4,
            bytes: 65536,
            out: 16,
            first: Duration::from_nanos(10),
            most: Duration::from_nanos(80),
            margin: Duration::from_secs(1),
        },
        fields: 4,
        reads: 4,
        parts: 8,
        delegates: 2,
        document_bytes: 65_536,
        records: 2,
        kept: 4,
        projects: 2,
        text: 32,
        segments: 4,
        rows: 2,
        backoff: jig_client_domain::Backoff { first: Duration::from_nanos(10), most: Duration::from_nanos(80) },
        heartbeat: Duration::from_nanos(100),
    };
    jig_client_protocol_link::Limits::new(client, 4).expect("valid receiving limits")
}
