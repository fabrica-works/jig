//! A current joint account/source/host check (accounts.md, 4.3; engine.md, 7.1).
use crate::{Core, RunPolicy};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_fleet as fleet;
use skein_lib::List;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RunWait {
    Account { account: Option<u32>, source: Option<u32>, why: accounts::WaitWhy },
    InvalidAccount { account: u32, why: jig_domain_tasks::AccountWhy },
    Credential,
    Host,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RunChoice {
    pub placement: fleet::PlacementConstraint,
    pub accounts: Box<[authority::AccountBinding]>,
    pub policy: RunPolicy,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RunAllowance {
    pub choice: RunChoice,
    pub spend: u64,
}

impl Core {
    /// Probing retains no state or grants. At most accounts × (hosts + 1) probes build
    /// at most one fact per source, joining each host's kept logins to issued holders.
    #[must_use = "retain the checked choice or its waiting reason"]
    #[expect(clippy::too_many_lines, reason = "one bounded joint account/source/host selection")]
    pub fn run_choice(&self, task: u64) -> Result<RunChoice, RunWait> {
        let row = self.tasks.task(task).ok_or(RunWait::Host)?;
        let payer = self.tasks.payer(task).ok_or(RunWait::Host)?;
        if row.kept_account.is_none()
            && let jig_domain_tasks::Funder::Recurring { .. } = row.funder
            && let Err(jig_domain_tasks::Refusal::Account { account, why }) =
                self.account_preference(row.llm.as_ref(), payer, None)
        {
            return Err(RunWait::InvalidAccount { account, why });
        }
        let coverage = self.account_coverage(payer);
        let policy = self.chosen_policy(task);
        let provider = self.provider(&policy.model).ok_or(account_wait(None))?;
        let preferred = row.kept_account.or(match &row.llm {
            Some(choice) => choice.account,
            None => None,
        });
        let mut excluded = None;
        if let Some(old) = row.kept_account
            && let Some(account) = self.accounts.account(old)
            && let Some(accounts::State::Limited { .. }) = self.accounts.account_state(old)
        {
            excluded = Some(accounts::Excluded { holder: account.holder(), provider: account.provider() });
        }
        let candidates =
            self.accounts.candidates(crate::coverage::payer(payer), provider, preferred, excluded, coverage.as_slice());
        let mut waiting = None;
        let mut host_wait = false;
        let kinds = self.charter_of(task).hosts;
        let main = crate::fleet_needs::model(&policy.model);
        let mut facts = List::with_capacity(self.host_fact_capacity());
        for account in &candidates {
            let mut tried = List::with_capacity(self.fleet.host_capacity());
            for _ in 0..=self.fleet.host_capacity() {
                let Some(placement) = self.fleet.probe(kinds, task, core::slice::from_ref(&main), tried.as_slice())
                else {
                    host_wait = true;
                    break;
                };
                self.host_facts(placement, self.host_rule(placement), &mut facts);
                let selection = self.accounts.select(
                    crate::coverage::payer(payer),
                    provider,
                    Some(*account),
                    None,
                    coverage.as_slice(),
                    facts.as_slice(),
                    self.run_now,
                );
                let source = match selection {
                    accounts::Selection::Chosen { source, .. } => source,
                    accounts::Selection::Wait { why: accounts::WaitWhy::Host, .. } => {
                        host_wait = true;
                        tried.push(placement.channel).expect("one rejected channel per host");
                        continue;
                    }
                    accounts::Selection::Wait { account, source, why } => {
                        if waiting.is_none() {
                            waiting = Some(RunWait::Account { account: Some(account), source, why });
                        }
                        break;
                    }
                    accounts::Selection::NoAccount => break,
                };
                host_wait = true;
                let mut bindings = List::with_capacity(jig_domain_tasks::MODELS);
                bindings.push(authority::AccountBinding { provider, account: *account, source }).expect("main binding");
                let mut retained = List::with_capacity(jig_domain_tasks::MODELS);
                for model in &policy.alternatives {
                    let Some(other) = self.provider(model) else { continue };
                    let mut bound = false;
                    for binding in &bindings {
                        if binding.provider == other {
                            bound = true;
                        }
                    }
                    if !bound
                        && let accounts::Selection::Chosen { account, source } = self.accounts.select(
                            crate::coverage::payer(payer),
                            other,
                            None,
                            None,
                            coverage.as_slice(),
                            facts.as_slice(),
                            self.run_now,
                        )
                    {
                        bindings
                            .push(authority::AccountBinding { provider: other, account, source })
                            .expect("one binding per retained provider");
                        bound = true;
                    }
                    if bound {
                        retained.push(model.clone()).expect("configured model bound");
                    }
                }
                let mut needs = List::with_capacity(jig_domain_tasks::MODELS.checked_add(1).expect("main need"));
                needs.push(main.clone()).expect("main need");
                for model in &retained {
                    needs.push(crate::fleet_needs::model(model)).expect("retained models");
                }
                if self.fleet.supports(placement, needs.as_slice()) {
                    let mut selected = policy.clone();
                    selected.alternatives = retained.into_boxed();
                    return Ok(RunChoice { placement, accounts: bindings.into_boxed(), policy: selected });
                }
                tried.push(placement.channel).expect("one rejected channel per host");
            }
        }
        if host_wait { Err(RunWait::Host) } else { Err(waiting.unwrap_or(account_wait(None))) }
    }

    pub(crate) fn provider(&self, model: &crate::Model) -> Option<u32> {
        Some(self.settings.llm_providers.serving(&model.name)?.number)
    }
}
fn account_wait(account: Option<u32>) -> RunWait {
    RunWait::Account { account, source: None, why: accounts::WaitWhy::Empty }
}

impl Core {
    /// Recheck recorded identities before a newly placed assignment leaves.
    #[must_use]
    pub(crate) fn claim_bindings_usable(&self, task: u64, attempt: u64) -> bool {
        let Some(proof) = self.proofs.get(&task) else { return false };
        if proof.attempt != attempt || proof.accounts.is_empty() {
            return false;
        }
        let Some(payer) = self.tasks.payer(task) else { return false };
        let coverage = self.account_coverage(payer);
        let channel = match proof.placement {
            Some(channel) => channel,
            None => match proof.host {
                fleet::HostKind::Engine => skein_lib::Token::new(0),
                fleet::HostKind::Worker => return false,
            },
        };
        let placement = fleet::PlacementConstraint { channel, kind: proof.host };
        if proof.host == fleet::HostKind::Worker && self.fleet.awaiting_logins(channel) {
            return false;
        }
        let mut facts = List::with_capacity(self.host_fact_capacity());
        self.host_facts(placement, self.host_rule(placement), &mut facts);
        for binding in &proof.accounts {
            let selected = self.accounts.select(
                crate::coverage::payer(payer),
                binding.provider,
                Some(binding.account),
                None,
                coverage.as_slice(),
                facts.as_slice(),
                self.run_now,
            );
            if selected != (accounts::Selection::Chosen { account: binding.account, source: binding.source }) {
                return false;
            }
        }
        self.credentials_ready(task)
    }
}
