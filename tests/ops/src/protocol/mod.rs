//! Results read in the world's protocol routing, after the fleet fence.
use jig_domain_tasks as tasks;
use jig_protocol_charter as charter;

#[must_use]
pub fn result(outcome: &[u8]) -> tasks::End {
    match read_result(outcome) {
        Some(result) => tasks::End::Finished { result, cancel_delegates: false },
        None => tasks::End::Failed(tasks::Class::Invalid),
    }
}

fn read_result(outcome: &[u8]) -> Option<tasks::TaskResult> {
    match charter::result(outcome).ok()? {
        charter::RunResult::Report { words } => Some(tasks::TaskResult::Report { words }),
        charter::RunResult::Failure { reason } => Some(tasks::TaskResult::Failure { reason }),
        charter::RunResult::Verdict { label, fields, words } => {
            let code = decimal(&label)?;
            Some(tasks::TaskResult::Verdict {
                code: u32::try_from(code).ok()?,
                words,
                fields: fields
                    .into_vec()
                    .into_iter()
                    .map(|field| tasks::ResultField { name: field.name, value: field.value })
                    .collect(),
            })
        }
    }
}

fn decimal(bytes: &[u8]) -> Option<u64> {
    if bytes.is_empty() {
        return None;
    }
    let mut number = 0_u64;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u64::from(*byte - b'0'))?;
    }
    Some(number)
}
