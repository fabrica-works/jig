//! Whole public provider offers and payer observations, bounded at the receiving face.
use super::{llm, owner};
use jig_client::DomainLimits as Limits;
use jig_client_domain::contract as c;
use jig_domain_accounts as accounts;
use jig_domain_core::documents as d;
use skein_lib::List;

pub(super) fn offers(
    values: &List<d::ProviderOffer>,
    limits: &Limits,
) -> Result<List<c::ProviderOffer>, owner::Invalid> {
    let wire = jig_client_protocol_link::Limits::new(*limits, 1).map_err(|_| owner::Invalid::TooLarge)?.wire;
    for (at, value) in values.iter().enumerate() {
        if value.key_bytes > wire.keyed_value_some {
            return Err(owner::Invalid::TooLarge);
        }
        for earlier in values.iter().take(at) {
            if earlier.provider == value.provider {
                return Err(owner::Invalid::Catalogue);
            }
        }
    }
    owner::list(values, limits, |value| {
        Ok(c::ProviderOffer {
            provider: value.provider,
            key_bytes: value.key_bytes,
            name: owner::text(&value.name, limits)?,
            sign_in: value.sign_in,
            key: value.key,
            programs: owner::list(&value.programs, limits, |value| {
                llm::name(value, &Limits { text: wire.provider_offer_programs_item, ..*limits })
            })?,
            programs_cut: value.programs_cut,
            none: value.none,
            signed_in_plan: value.signed_in_plan,
            manage_usage: manage_usage(value.manage_usage.as_deref())?,
        })
    })
}

pub(super) fn manage_usage(value: Option<&[u8]>) -> Result<Option<Box<[u8]>>, owner::Invalid> {
    let Some(value) = value else { return Ok(None) };
    if !c::valid_manage_usage(value) {
        return Err(owner::Invalid::Catalogue);
    }
    Ok(Some(value.into()))
}

pub(super) fn readiness(
    values: &List<d::PayerReadiness>,
    limits: &Limits,
) -> Result<List<c::PayerReadiness>, owner::Invalid> {
    for (at, value) in values.iter().enumerate() {
        for earlier in values.iter().take(at) {
            if earlier.provider == value.provider && earlier.payer == value.payer {
                return Err(owner::Invalid::Catalogue);
            }
        }
    }
    owner::list(values, limits, |value| {
        Ok(c::PayerReadiness {
            provider: value.provider,
            payer: match value.payer {
                accounts::Use::Party(value) => c::OwnerAccountUse::Party(value),
                accounts::Use::Project(value) => c::OwnerAccountUse::Project(value),
            },
            readiness: match value.readiness {
                d::Readiness::Usable => c::Readiness::Usable,
                d::Readiness::Unavailable => c::Readiness::Unavailable,
                d::Readiness::Unknown => c::Readiness::Unknown,
            },
            settled: value.settled,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offer() -> d::ProviderOffer {
        d::ProviderOffer {
            provider: 7,
            key_bytes: 123,
            name: d::Text { bytes: Box::from(b"Provider".as_slice()), cut: true },
            sign_in: true,
            key: false,
            programs: crate::test::list([Box::from(b"program".as_slice())]),
            programs_cut: true,
            none: false,
            signed_in_plan: true,
            manage_usage: Some(Box::from(b"https://usage.example/".as_slice())),
        }
    }

    fn observations() -> List<d::PayerReadiness> {
        crate::test::list([
            d::PayerReadiness {
                provider: 7,
                payer: accounts::Use::Party(99),
                readiness: d::Readiness::Usable,
                settled: Some(true),
            },
            d::PayerReadiness {
                provider: 8,
                payer: accounts::Use::Project(1),
                readiness: d::Readiness::Unavailable,
                settled: Some(false),
            },
            d::PayerReadiness {
                provider: 9,
                payer: accounts::Use::Party(99),
                readiness: d::Readiness::Unknown,
                settled: None,
            },
        ])
    }

    #[test]
    fn me_carries_public_provider_offers() {
        let mut source = super::super::documents::tests::frame();
        source.providers = crate::test::list([offer()]);
        source.providers_cut = true;
        source.key_bytes = 123;
        let value =
            owner::frame(&source, &super::super::deployment::tests::limits(), &[], &accounts::Providers::none())
                .expect("public Me");
        assert_eq!(value.key_bytes, 123);
        assert!(value.providers_cut);
        assert_eq!(
            value.providers.as_slice(),
            &[c::ProviderOffer {
                provider: 7,
                key_bytes: 123,
                name: c::DocumentText {
                    bytes: c::Text::new(Box::from(b"Provider".as_slice()), 200).expect("name"),
                    cut: true
                },
                sign_in: true,
                key: false,
                programs: crate::test::list([c::Text::new(Box::from(b"program".as_slice()), 200).expect("program")]),
                programs_cut: true,
                none: false,
                signed_in_plan: true,
                manage_usage: Some(Box::from(b"https://usage.example/".as_slice())),
            }]
        );
    }

    #[test]
    fn deployment_carries_payer_observations() {
        let mut source = super::super::deployment::tests::source();
        source.providers = crate::test::list([offer()]);
        source.providers_cut = true;
        source.key_bytes = 123;
        source.readiness = observations();
        source.readiness_cut = true;
        let limits = super::super::deployment::tests::limits();
        let value = super::super::deployment::document(&source, &limits).expect("whole deployment");
        assert_eq!(value.providers, offers(&source.providers, &limits).expect("same public catalogue"));
        assert_eq!(value.key_bytes, 123);
        assert!(value.providers_cut && value.readiness_cut);
        assert_eq!(
            value.readiness.as_slice(),
            &[
                c::PayerReadiness {
                    provider: 7,
                    payer: c::OwnerAccountUse::Party(99),
                    readiness: c::Readiness::Usable,
                    settled: Some(true)
                },
                c::PayerReadiness {
                    provider: 8,
                    payer: c::OwnerAccountUse::Project(1),
                    readiness: c::Readiness::Unavailable,
                    settled: Some(false)
                },
                c::PayerReadiness {
                    provider: 9,
                    payer: c::OwnerAccountUse::Party(99),
                    readiness: c::Readiness::Unknown,
                    settled: None
                },
            ]
        );
    }

    #[test]
    fn accounts_carry_payer_observations() {
        let source = d::Accounts {
            accounts: List::with_capacity(0),
            signing: List::with_capacity(0),
            last_end: None,
            usable: true,
            settled: false,
            cut: false,
            readiness: observations(),
            readiness_cut: true,
        };
        let limits = super::super::deployment::tests::limits();
        let value = super::super::accounts::document(&source, &limits).expect("whole accounts");
        assert_eq!(value.readiness, readiness(&source.readiness, &limits).expect("same observations"));
        assert!(value.readiness_cut);
    }

    #[test]
    fn provider_allocations_obey_the_receiving_bounds() {
        let values = crate::test::list([offer()]);
        let limits = super::super::deployment::tests::limits();
        assert!(offers(&values, &Limits { rows: 1, ..limits }).is_ok());
        assert_eq!(offers(&values, &Limits { rows: 0, ..limits }), Err(owner::Invalid::TooLarge));
        assert_eq!(offers(&values, &Limits { text: 6, ..limits }), Err(owner::Invalid::TooLarge));
        let mut oversized = offer();
        oversized.programs = List::with_capacity(2);
        assert_eq!(
            offers(&crate::test::list([oversized]), &Limits { rows: 1, ..limits }),
            Err(owner::Invalid::TooLarge)
        );
    }
    #[test]
    fn duplicate_providers_are_refused() {
        assert_eq!(
            offers(&crate::test::list([offer(), offer()]), &super::super::deployment::tests::limits()),
            Err(owner::Invalid::Catalogue)
        );
    }

    #[test]
    fn provider_key_sizes_obey_the_wire_bound() {
        let limits = super::super::deployment::tests::limits();
        let maximum = jig_client_protocol_link::Limits::new(limits, 1).expect("wire bounds").wire.keyed_value_some;
        let results = [maximum, maximum + 1].map(|key_bytes| {
            let mut value = offer();
            value.key_bytes = key_bytes;
            offers(&crate::test::list([value]), &limits).map(|_| ())
        });
        assert_eq!(results, [Ok(()), Err(owner::Invalid::TooLarge)]);
    }

    #[test]
    fn duplicate_payer_observations_are_refused() {
        let first = *observations().get(0).expect("payer observation");
        let mut second = first;
        second.readiness = d::Readiness::Unknown;
        second.settled = None;
        assert_eq!(
            readiness(&crate::test::list([first, second]), &super::super::deployment::tests::limits()),
            Err(owner::Invalid::Catalogue)
        );
    }

    #[test]
    fn unsafe_manage_targets_are_refused() {
        for target in [b"http://usage.example/".as_slice(), b"https://user@usage.example/".as_slice()] {
            let mut value = offer();
            value.manage_usage = Some(target.into());
            assert_eq!(
                offers(&crate::test::list([value]), &super::super::deployment::tests::limits()),
                Err(owner::Invalid::Catalogue)
            );
        }
    }

    #[test]
    fn program_names_obey_the_wire_bound() {
        let limits = super::super::deployment::tests::limits();
        let mut value = offer();
        value.programs = crate::test::list([Box::<[u8]>::from([b'a'; 32])]);
        assert!(offers(&crate::test::list([value.clone()]), &limits).is_ok());
        value.programs = crate::test::list([Box::<[u8]>::from([b'a'; 33])]);
        assert_eq!(offers(&crate::test::list([value]), &limits), Err(owner::Invalid::TooLarge));
    }
}
