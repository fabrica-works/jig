use super::config;
use jig_domain_tasks::Contract;
const SEED: u64 = 513;
const READY_REPORT: &[u8] = b"staging/payments is ready for the requested week";
#[test]
fn the_environment_chat_template_fits_its_supplied_ready_report() {
    let (config, _) = config::root_for(SEED, true);
    let template = config
        .core
        .defaults
        .templates
        .iter()
        .find(|row| row.flags.conversation)
        .expect("fixture has a conversation template");
    let Contract::Report { words } = template.contract else { panic!("fixture uses a report") };
    assert!(
        READY_REPORT.len() <= usize::try_from(words).expect("report limit fits usize"),
        "the supplied {} byte report exceeds its template's {words} byte contract",
        READY_REPORT.len()
    );
}
