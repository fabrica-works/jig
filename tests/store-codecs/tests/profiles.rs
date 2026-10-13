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
