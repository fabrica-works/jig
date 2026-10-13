//! Shipped profiles fit the byte-summed families (store.md, 5.1).
use jig_protocol_config::{Profile, profile};
use jig_protocol_store::codec;

#[test]
fn both_shipped_profiles_admit_the_tasks_and_people_families() {
    for profile_name in [Profile::Small, Profile::Standard] {
        let limits = profile(profile_name);
        assert!(codec::tasks::limits(&limits.core.tasks).is_some(), "{profile_name:?} Tasks");
        assert!(codec::people::limits(&limits.core).is_some(), "{profile_name:?} People");
    }
}

#[test]
fn both_shipped_profiles_round_trip_the_settled_call_sum() {
    use jig_domain_core as core;
    let answers = [
        ("host", core::SettledAnswer::Host { error: false, body: vec![1; 16_384].into_boxed_slice() }),
        (
            "delivery",
            core::SettledAnswer::Delivery {
                outcome: core::DeliveryOutcome::Delivered,
                evidence: vec![2; 16_384].into_boxed_slice(),
            },
        ),
    ];
    for profile_name in [Profile::Small, Profile::Standard] {
        let domain = profile(profile_name);
        let limits = codec::limits(&domain.core).expect("shipped profile within codec ceilings");
        assert_eq!(domain.core.call_answer_bytes, 32_768, "{profile_name:?}");
        let core_limits = codec::core::Limits { core: limits.core, tasks: limits.tasks, notes: limits.notes };
        for (label, answer) in &answers {
            let row = core::CoreRecord::Call(core::CallRecord {
                key: core::CallKey { task: 1, attempt: 2, completion: 3, position: 4 },
                part: core::CallPart::Unavailable,
                settled: Some(core::SettledCall {
                    serial: 1,
                    name: Box::from(b"n".as_slice()),
                    tool: vec![3; 16_383].into_boxed_slice(),
                    answer: answer.clone(),
                }),
            });
            let bytes = codec::core::encode(&row, &core_limits).expect("sum at its limit");
            assert_eq!(codec::core::decode(&bytes, &core_limits), Ok(row), "{profile_name:?} {label}");
        }
    }
}
