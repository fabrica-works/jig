//! Pages of another origin send the forged requests from the person's browser.
use crate::Target;
use base64::{Engine, engine::general_purpose::STANDARD};
use skein_fake_person_browser::origins::Page;

/// Two forged fetches; the harness substitutes the application's `{address}`.
#[must_use]
pub fn forging(target: &Target<'_>) -> Page {
    let request = STANDARD.encode(target.request);
    let prefix = std::str::from_utf8(target.prefix).expect("mapping prefix is UTF-8");
    let body = format!(
        r#"<!doctype html><title>Elsewhere</title><script>
const body = Uint8Array.from(atob("{request}"), c => c.charCodeAt(0));
fetch("http://{{address}}{prefix}/v1/request", {{method: "POST", mode: "no-cors", credentials: "include", headers: {{"Content-Type": "text/plain"}}, body}}).catch(() => {{}});
fetch("http://{{address}}{prefix}/v1/request", {{method: "POST", credentials: "include", headers: {{"Content-Type": "application/octet-stream", "Jig-Client": "browser"}}, body}}).catch(() => {{}});
</script>
"#
    );
    Page {
        path: b"/forging".as_slice().into(),
        kind: b"text/html".as_slice().into(),
        body: body.into_bytes().into(),
        headers: vec![],
    }
}
