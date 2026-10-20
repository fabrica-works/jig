//! Explicit account admission uses the prospective financial payer, never the requester
//! (accounts.md, 4.1–4.3; authority.md, 8.1; tasks.md, section 7).
use crate::Core;
use jig_domain_accounts as accounts;
use jig_domain_tasks as tasks;

impl Core {
    /// Availability is deliberately absent: a valid unavailable account is accepted and waits.
    pub(crate) fn account_preference(
        &self,
        choice: Option<&tasks::LlmChoice>,
        payer: tasks::Payer,
        kept: Option<u32>,
    ) -> Result<(), tasks::Refusal> {
        let Some(choice) = choice else { return Ok(()) };
        let Some(number) = choice.account else { return Ok(()) };
        let model = self
            .settings
            .models
            .get(usize::from(choice.main.model))
            .ok_or(tasks::Refusal::Llm { model: choice.main.model, why: tasks::LlmFault::Unconfigured })?;
        let account = self.accounts.account(number).ok_or(refused(number, tasks::AccountWhy::Unknown))?;
        let provider = self.provider(model).ok_or(refused(number, tasks::AccountWhy::Provider))?;
        if account.provider() != provider {
            return Err(refused(number, tasks::AccountWhy::Provider));
        }
        if !crate::coverage::serves(&self.people, account, crate::coverage::payer(payer)) {
            return Err(refused(number, tasks::AccountWhy::Uses));
        }
        if let Some(old) = kept
            && old != number
            && let Some(kept) = self.accounts.account(old)
            && kept.provider() == provider
        {
            for fact in &self.accounts.account_views() {
                if fact.account == old
                    && let accounts::State::Limited { .. } = fact.state
                {
                    return Err(refused(number, tasks::AccountWhy::Limited));
                }
            }
        }
        Ok(())
    }

    /// The same account check before a standalone or accepted amendment reaches tasks.
    pub(crate) fn amendment_account(
        &self,
        task: u64,
        amendment: &tasks::Amendment,
        funder: Option<tasks::Funder>,
    ) -> Result<(), tasks::Refusal> {
        let Some(choice) = &amendment.llm else { return Ok(()) };
        if choice.account.is_none() {
            return Ok(());
        }
        let row = self.tasks.task(task).ok_or(tasks::Refusal::Unknown)?;
        let payer = self.tasks.prospective_payer(funder.unwrap_or(row.funder), &[]).ok_or(tasks::Refusal::Funding)?;
        self.account_preference(Some(choice), payer, row.kept_account)
    }
}

fn refused(account: u32, why: tasks::AccountWhy) -> tasks::Refusal {
    tasks::Refusal::Account { account, why }
}

impl Core {
    pub(crate) fn account_refusal(&self, why: tasks::Refusal) -> jig_domain_people::Refusal {
        let jig_domain_people::Outcome::Refused(why) = crate::routing::refusals::person_task_refusal(self, None, why)
        else {
            unreachable!("account admission has a permanent refusal");
        };
        why
    }

    /// All members of a funded proposal take the accepter's financial source.
    pub(crate) fn funded_accounts(&self, batch: &[tasks::New], funder: tasks::Funder) -> Result<(), tasks::Refusal> {
        for member in batch {
            if let Some(choice) = &member.llm
                && choice.account.is_some()
            {
                let payer = self.tasks.prospective_payer(funder, &[]).ok_or(tasks::Refusal::Funding)?;
                self.account_preference(Some(choice), payer, None)?;
            }
        }
        Ok(())
    }

    pub(crate) fn action_accounts(
        &self,
        action: &tasks::ProposalAction,
        funder: tasks::Funder,
    ) -> Result<(), tasks::Refusal> {
        match action {
            tasks::ProposalAction::Batch(batch) => self.funded_accounts(batch, funder),
            tasks::ProposalAction::Amend { task, amendment, .. } => {
                self.amendment_account(*task, amendment, Some(funder))
            }
            tasks::ProposalAction::Effect { .. }
            | tasks::ProposalAction::ProcedureEffect { .. }
            | tasks::ProposalAction::Release { .. }
            | tasks::ProposalAction::Widen { .. } => Ok(()),
        }
    }
}
