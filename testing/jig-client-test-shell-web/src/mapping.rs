//! The test server mounts the mapping under /jig.
pub fn mapping() -> jig_client_protocol_http::Mapping {
    jig_client_protocol_http::Mapping::new(b"/jig").expect("mapping prefix")
}
