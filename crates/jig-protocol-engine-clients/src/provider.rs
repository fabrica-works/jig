//! `OpenID` authorization before credential checking (protocol/client.md, 18).
use crate::{Limits, Reply, exchange::Exchange, routes};
use jig_client_protocol_wire::v18 as w;
use skein_lib::{Env, Reader};
pub(super) fn authorize(env: &Env<Limits>, connection: &mut Exchange, body: &[u8]) {
    let mut reader = Reader::new(body);
    if w::Authorize::decode(&env.limits.wire, &mut reader).is_err() || !reader.is_empty() {
        connection.respond(Reply::NotARecord, &env.limits);
        return;
    }
    let answer = w::AuthorizeAnswer::Refused(
        w::AuthorizeRefused::new(&env.limits.wire, w::AuthorizeRefusedParts { why: w::AuthorizeWhy::NoProvider })
            .expect("fixed authorization refusal"),
    );
    connection.respond(Reply::Record(routes::authorize_answer(answer)), &env.limits);
}
