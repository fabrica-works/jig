use crate::{Limits, Profile, Share, What, fits, profile};

fn shares() -> [Share; 2] {
    [Share { name: b"core", bytes: 600 }, Share { name: b"hub", bytes: 500 }]
}
fn limits(profile_name: Profile) -> Limits {
    profile(profile_name)
}
fn assert_children_fit(profile_name: Profile) {
    let core = limits(profile_name).core;
    let bounds = [
        ("tasks", jig_domain_tasks::worst_case(&core.tasks)),
        ("authority", jig_domain_authority::worst_case(&core.authority)),
        ("people", jig_domain_people::worst_case(&core.people)),
        ("fleet", jig_domain_fleet::worst_case(&core.fleet)),
        ("brief", jig_domain_brief::gather_worst_case(&core.brief)),
        ("accounts", jig_domain_accounts::worst_case(&core.accounts)),
        ("notes", jig_domain_notes::worst_case(&core.notes)),
        ("views", jig_domain_views::worst_case(&core.views)),
    ];
    for (name, bytes) in bounds {
        assert!(bytes.is_some(), "{profile_name:?} {name} has a representable worst case");
    }
}
fn core_bytes(profile_name: Profile) -> Option<u64> {
    jig_domain_core::worst_case(&limits(profile_name).core)
}
fn hub_bytes(profile_name: Profile) -> Option<u64> {
    jig_domain_host::worst_case(&limits(profile_name).hub)
}

#[test]
fn a_stated_memory_below_the_sum_is_refused_by_how_much() {
    let shares = shares();
    assert_eq!(fits(1_000, &shares), Err(What::Memory { over: 100 }));
}
#[test]
fn the_small_profile_fits_every_childs_worst_case() {
    assert_children_fit(Profile::Small);
}
#[test]
fn the_standard_profile_fits_every_childs_worst_case() {
    assert_children_fit(Profile::Standard);
}
#[test]
fn the_small_profile_fits_the_cores_worst_case() {
    assert!(core_bytes(Profile::Small).is_some(), "small core bound");
}
#[test]
fn the_standard_profile_fits_the_cores_worst_case() {
    assert!(core_bytes(Profile::Standard).is_some(), "standard core bound");
}
#[test]
fn the_small_profiles_hub_fits_its_worst_case() {
    assert!(hub_bytes(Profile::Small).is_some(), "small hub bound");
}
#[test]
fn the_standard_profiles_hub_fits_its_worst_case() {
    assert!(hub_bytes(Profile::Standard).is_some(), "standard hub bound");
}
#[test]
fn a_sum_past_a_u64_is_refused() {
    let shares = [Share { name: b"core", bytes: u64::MAX }, Share { name: b"hub", bytes: 1 }];
    assert_eq!(fits(u64::MAX, &shares), Err(What::Bound));
}
#[test]
fn a_sum_within_memory_is_its_total() {
    let shares = shares();
    assert_eq!(fits(1_200, &shares), Ok(1_100));
}
