use super::*;
use skein_lib::List;
pub fn owner_deployment_record() -> w::OwnerDeployment {
    jig_client_test_records::owner_deployment_record()
}
pub fn owner_deployment() -> c::OwnerDeployment {
    c::OwnerDeployment {
        store: c::StoreFill { bytes: 7, mark: 7 },
        hosts: List::with_capacity(0),
        attempts: List::with_capacity(0),
        accounts: List::with_capacity(0),
        tasks: c::OwnerCapacity { used: 7, limit: 7 },
        workers: c::OwnerCapacity { used: 7, limit: 7 },
        attempts_capacity: c::OwnerCapacity { used: 7, limit: 7 },
        waiting_attempts: 7,
        proposals: c::OwnerCapacity { used: 7, limit: 7 },
        projects: List::with_capacity(0),
        grace_bound: Duration::from_nanos(19),
        hello_refusals: List::with_capacity(0),
        hosts_forgotten: 7,
        hello_refusals_forgotten: 7,
        person_room_refusals: List::with_capacity(0),
        person_room_refusals_forgotten: 7,
        person_answer_retention: Duration::from_nanos(19),
        task_room_refusals: List::with_capacity(0),
        task_room_refusals_forgotten: 7,
        cut: false,
        connectors: List::with_capacity(0),
        connectors_cut: false,
    }
}
pub fn owner_project_record() -> w::OwnerProject {
    jig_client_test_records::owner_project_record()
}
pub fn owner_project() -> c::OwnerProject {
    c::OwnerProject {
        acts: List::with_capacity(0),
        number: 7,
        resources: List::with_capacity(0),
        policy: c::OwnerProjectPolicy {
            templates: List::with_capacity(0),
            ceiling: c::OwnerAuthorityDocument {
                authority: c::OwnerAuthority {
                    models: Some(c::AuthorityModels::All),
                    tools: 7,
                    grants: List::with_capacity(0),
                    delegation: c::OwnerDelegation { kinds: List::with_capacity(0), tasks: 7, depth: 7 },
                    spend: 7,
                    deadline: None,
                    notes: 7,
                    note_resources: List::with_capacity(0),
                },
                cut: false,
            },
            period_spend: 7,
            escalation_role: None,
            projections: List::with_capacity(0),
            roles: List::with_capacity(0),
            requirements: List::with_capacity(0),
            permissions: None,
            permissions_cut: false,
            cut: false,
        },
        members: List::with_capacity(0),
        current_period: None,
        pools: List::with_capacity(0),
        configured_period: 7,
        configured_pools: List::with_capacity(0),
        configured_pools_cut: false,
        spend: List::with_capacity(0),
        recurring: List::with_capacity(0),
        cut: false,
    }
}
pub fn owner_with_account(until: Option<u64>) -> w::OwnerDeployment {
    let lim = limits().wire;
    let account = w::OwnerAccount::new(
        &lim,
        w::OwnerAccountParts {
            provider_name: None,
            manage_usage: None,
            skein_bools: [false, true, false, true, false],
            number: 1,
            place: 0,
            uses: None,
            provider: None,
            display: w::DocumentText::new(&lim, w::DocumentTextParts { bytes: Box::from([]), cut: false }).unwrap(),
            holder: w::OwnerAccountHolder::Deployment,
            sources: Box::from([]),
            generation: Some(0),
            state: w::OwnerAccountState::Later(
                w::OwnerAccountStateLater::new(&lim, w::OwnerAccountStateLaterParts { until }).unwrap(),
            ),
            spent_until: None,
            runs: Box::from([]),
            waiting: Box::from([]),
        },
    )
    .unwrap();
    let mut bytes = Writer::new(usize::try_from(account.measure()).unwrap());
    account.encode(&mut bytes).unwrap();
    let bytes = bytes.finish();
    let mut piece = Writer::new(bytes.len().checked_add(4).unwrap());
    skein_codec::piece::write(&mut piece, &bytes).unwrap();
    let mut parts = owner_deployment_record().into_parts();
    parts.accounts = piece.finish();
    w::OwnerDeployment::new(&lim, parts).unwrap()
}
pub fn owner_account_expected(until: Option<u64>) -> c::OwnerDeployment {
    let until = match until {
        Some(0) => Some(skein_lib::Wall::from_nanos(0)),
        Some(value) => Some(skein_lib::Wall::from_nanos(value)),
        None => None,
    };
    let mut owner = owner_deployment();
    owner.accounts = List::with_capacity(1);
    owner
        .accounts
        .push(c::OwnerAccount {
            provider_name: None,
            manage_usage: None,
            number: 1,
            place: 0,
            uses: None,
            plan: false,
            provider: None,
            display: c::DocumentText { bytes: c::Text::new(Box::from([]), 32).unwrap(), cut: false },
            holder: c::OwnerAccountHolder::Deployment,
            sources: List::with_capacity(0),
            generation: Some(0),
            state: c::OwnerAccountState::Later { until },
            usable: true,
            expiring: false,
            attention: true,
            spent_until: None,
            runs: List::with_capacity(0),
            waiting: List::with_capacity(0),
            cut: false,
        })
        .unwrap();
    owner
}
