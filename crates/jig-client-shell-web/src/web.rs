//! The platform half; transport and same-origin fetch are skein's (protocol/browser.md, 5.1).
use crate::{
    look::Phrases,
    sign_in::{self, Route},
};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

/// Rewrite history before handing the secret to the shell's host.
#[must_use]
pub fn take_from_page(route: &Route<'_>) -> Option<Box<[u8]>> {
    let window = web_sys::window().expect("browser window");
    let location = window.location();
    let address = location.pathname().expect("page path") + &location.search().expect("page query");
    let taken = sign_in::take(route, address.as_bytes())?;
    window
        .history()
        .expect("history")
        .replace_state_with_url(&JsValue::NULL, "", Some(std::str::from_utf8(&taken.address).expect("home is UTF-8")))
        .expect("rewrite the link before starting the host");
    Some(taken.secret)
}
/// The credential lives only in this selected field, cleared and removed when closed.
pub fn show_once(words: Phrases<'_>, credential: &[u8]) {
    let document = web_sys::window().expect("window").document().expect("document");
    let dialog = document
        .create_element("dialog")
        .expect("dialog")
        .dyn_into::<web_sys::HtmlDialogElement>()
        .expect("dialog element");
    dialog.set_attribute("aria-label", words.credential).expect("dialog name");
    let instruction = document.create_element("p").expect("instruction");
    instruction.set_text_content(Some(words.keep));
    dialog.append_child(&instruction).expect("dialog instruction");
    let field = document
        .create_element("textarea")
        .expect("credential field")
        .dyn_into::<web_sys::HtmlTextAreaElement>()
        .expect("text area");
    field.set_read_only(true);
    field.set_attribute("aria-label", words.credential).expect("credential name");
    let size = skein_lib::base64::encoded_len(
        skein_lib::base64::Alphabet::Url,
        u32::try_from(credential.len()).expect("credential length fits"),
    )
    .expect("encoded credential fits");
    let mut encoded = skein_lib::Writer::new(usize::try_from(size).expect("encoded length fits"));
    skein_lib::base64::encode(skein_lib::base64::Alphabet::Url, credential, &mut encoded).expect("measured credential");
    field.set_value(std::str::from_utf8(&encoded.finish()).expect("base64 is UTF-8"));
    dialog.append_child(&field).expect("credential field");
    let form = document.create_element("form").expect("closing form");
    form.set_attribute("method", "dialog").expect("close the dialog");
    let button = document.create_element("button").expect("close button");
    button.set_text_content(Some(words.close));
    form.append_child(&button).expect("closing button");
    dialog.append_child(&form).expect("closing form");
    document.body().expect("body").append_child(&dialog).expect("show the credential outside the tree");
    let removed = dialog.clone();
    let selected = field.clone();
    let closed = Closure::once_into_js(move |_: web_sys::Event| {
        selected.set_value("");
        removed.remove();
    });
    dialog.add_event_listener_with_callback("close", closed.unchecked_ref()).expect("clear on closing");
    dialog.show_modal().expect("credential dialog opens");
    field.focus().expect("credential focus");
    field.select();
}
