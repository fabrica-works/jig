use super::*;
use skein_lib::List;

fn names(values: &[&[u8]]) -> c::AuthorityModels {
    let mut names = List::with_capacity(u32::try_from(values.len()).unwrap());
    for value in values {
        names.push((*value).into()).unwrap();
    }
    c::AuthorityModels::Named(names)
}
#[test]
fn selectors_and_cut_authorities_round_trip_without_becoming_replacements() {
    let limits = fixtures::limits();
    for models in [None, Some(c::AuthorityModels::All), Some(names(&[])), Some(names(&[b"second", b"first"]))] {
        let mut authority = fixtures::read_value_owner_authority();
        authority.models = models;
        let wire = translate::asks::wire_owner_authority(&authority, &limits.wire).unwrap();
        assert_eq!(translate::documents::owner_authority_fields(&wire, &limits), Ok(authority.clone()));
        let mut common = fixtures::read_wire_authority().into_parts();
        common.models = wire.models().clone();
        let common = w::Authority::new(&limits.wire, common).unwrap();
        assert_eq!(translate::pages::authority_fields(&common, &limits).unwrap().models, authority.models);

        let document =
            w::OwnerAuthorityDocument::new(&limits.wire, w::OwnerAuthorityDocumentParts { authority: wire, cut: true })
                .unwrap();
        let read = translate::documents::owner_authority_document_fields(&document, &limits).unwrap();
        assert!(read.cut);
        assert_eq!(read.whole(), None);
        assert_eq!(read.authority, authority);
    }
}
#[test]
fn malformed_and_one_past_model_sets_are_refused_and_exact_bounds_fit() {
    let base = fixtures::limits();
    let limits =
        Limits::new(jig_client_domain::Limits { text: 4096, turn_text: 4096, ..base.client }, base.exchanges).unwrap();
    let mut authority = fixtures::read_value_owner_authority();
    for bad in [names(&[b""]), names(&[b"same", b"same"]), names(&[b"\xff"]), names(&[&[b'x'; 4097]])] {
        authority.models = Some(bad);
        translate::asks::wire_owner_authority(&authority, &limits.wire).expect_err("invalid selector");
    }
    for count in [64_u32, 65] {
        let mut values = List::with_capacity(count);
        for at in 0..count {
            values
                .push(Box::from([b'0'.checked_add(u8::try_from(at).expect("fixture index")).expect("ASCII name")]))
                .unwrap();
        }
        authority.models = Some(c::AuthorityModels::Named(values));
        assert_eq!(translate::asks::wire_owner_authority(&authority, &limits.wire).is_ok(), count == 64);
    }
    authority.models = Some(names(&[&[b'x'; 4096]]));
    translate::asks::wire_owner_authority(&authority, &limits.wire).expect("exact name bound");
}
