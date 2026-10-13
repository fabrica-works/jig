//! The test workspace answers locally; its owner names the prepared workspace.
use alloc::boxed::Box;
use jig_domain_host as host;
use skein_lib::{Map, Token};
use smith_host_domain as smith;

#[derive(Debug)]
pub struct Content {
    pub title: Box<[u8]>,
    pub body: Box<[u8]>,
}

#[derive(Debug)]
pub(crate) struct Workspace {
    pub(crate) contents: Map<Token, Option<Content>>,
}

impl Workspace {
    pub(crate) fn new(slots: u32) -> Self {
        Self { contents: Map::with_capacity(slots) }
    }

    pub(crate) fn prepare(&mut self, owner: Token) -> host::Event {
        let previous = self.contents.insert(owner, None).expect("the hub bounds prepared workspaces");
        assert!(previous.is_none(), "one prepare per hosted owner");
        host::Event::Prepared { owner, workspace: owner }
    }

    pub(crate) fn deliver(&mut self, workspace: Token, title: Box<[u8]>, body: Box<[u8]>) -> smith::Delivery {
        let content = self.contents.get_mut(&workspace).expect("prepared workspace");
        *content = Some(Content { title, body });
        let receipt = smith::Receipt::new(0, Box::new(workspace.raw().to_be_bytes())).expect("fixed opaque receipt");
        smith::Delivery::Delivered(smith::Delivered::new(Box::new([receipt])).expect("one receipt"))
    }

    pub(crate) fn release(&mut self, workspace: Token) {
        drop(self.contents.remove(&workspace));
    }
}
