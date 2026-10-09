//! Accessibility roles and owned bytes, with no layout or application types.
//! Nodes are preorder; a subtree occupies `size` consecutive nodes including
//! its root (client/domain/view.md, section 3). New builds have unassigned ids;
//! the diff assigns ids when the tree becomes visible.

use alloc::boxed::Box;
use skein_lib::{List, Token};

/// A visible node's identity, preserved while it matches across builds.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct NodeId(pub u32);

/// The view child that interprets a token.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Owner {
    /// jig's own view.
    Jig,
    /// A connector's client side.
    Side { connector: u16 },
    /// Another application child.
    Other { child: u16 },
}

/// A node's meaning, mapped exhaustively by every shell.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Role {
    /// A page node.
    Page,
    /// A region node.
    Region,
    /// A heading at a semantic level.
    Heading { level: u8 },
    /// A paragraph node.
    Paragraph,
    /// A text node.
    Text,
    /// A list node.
    List,
    /// A item node.
    Item,
    /// A table node.
    Table,
    /// A row node.
    Row,
    /// A cell node.
    Cell,
    /// A term node.
    Term,
    /// A description node.
    Description,
    /// A button node.
    Button,
    /// A link node.
    Link,
    /// A textfield node.
    TextField,
    /// A textarea node.
    TextArea,
    /// A choice node.
    Choice,
    /// A choicegroup node.
    ChoiceGroup,
    /// A check node.
    Check,
    /// A dialog node.
    Dialog,
    /// A status node.
    Status,
    /// A alert node.
    Alert,
    /// A log node.
    Log,
    /// A tree node.
    Tree,
    /// A treeitem node.
    TreeItem,
    /// A tabs node.
    Tabs,
    /// A tab node.
    Tab,
    /// A progress node.
    Progress,
    /// A time node.
    Time,
    /// A code node.
    Code,
    /// A quote node.
    Quote,
    /// A separator node.
    Separator,
    /// A figure node.
    Figure,
    /// A slot node.
    Slot,
    /// A drawing node.
    Drawing,
}

/// A semantic state, never a visual style.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum State {
    /// The node is disabled.
    Disabled,
    /// The node is expanded.
    Expanded,
    /// The node is busy.
    Busy,
    /// The node is current.
    Current,
    /// The node is invalid.
    Invalid,
    /// The node is pressed.
    Pressed,
    /// The node is selected.
    Selected,
    /// The node is provisional.
    Provisional,
    /// The node is submitsonenter.
    SubmitsOnEnter,
}

/// A token from an owner's closed style set.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Style {
    pub owner: Owner,
    pub token: u16,
}

/// The meaning of a span of text.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Run {
    /// Ordinary words.
    Plain,
    /// Emphasized words.
    Emphasis,
    /// Strongly emphasized words.
    Strong,
    /// Literal code.
    Code,
    /// A target interpreted by the text's owner.
    Link { to: Token },
}

/// One owned span on a text node.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TextRun {
    pub run: Run,
    pub text: Box<[u8]>,
}

/// A field's value and count of the domain's own writes.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Value {
    pub text: Box<[u8]>,
    pub written: u64,
}

/// The object an item represents, independent of its sibling position.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Key {
    pub owner: Owner,
    pub kind: u16,
    pub a: u64,
    pub b: u64,
}

/// A fragment requested from its owner's view.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SlotRef {
    pub owner: Owner,
    pub token: Token,
}

/// An owner's drawing vocabulary number.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DrawingKind {
    pub owner: Owner,
    pub kind: u16,
}

/// Opaque drawing data; ordinary child nodes provide its readable fallback.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Drawing {
    pub kind: DrawingKind,
    pub data: Box<[u8]>,
}

/// One node and its attributes; `size` includes its descendants.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Node {
    pub id: NodeId,
    pub role: Role,
    pub size: u32,
    pub name: Option<Box<[u8]>>,
    pub states: List<State>,
    pub styles: List<Style>,
    pub text: List<TextRun>,
    pub value: Option<Value>,
    pub link: Option<crate::Binding>,
    pub key: Option<Key>,
    pub binding: Option<crate::Binding>,
    pub slot: Option<SlotRef>,
    pub drawing: Option<Drawing>,
}

impl Node {
    pub(crate) fn vacant() -> Node {
        Node {
            id: NodeId(0),
            role: Role::Text,
            size: 1,
            name: None,
            states: List::with_capacity(0),
            styles: List::with_capacity(0),
            text: List::with_capacity(0),
            value: None,
            link: None,
            key: None,
            binding: None,
            slot: None,
            drawing: None,
        }
    }

    pub(crate) fn new(role: Role, limits: &crate::Limits) -> Node {
        let runs = if role == Role::Text { limits.text_per_node } else { 0 };
        Node {
            id: NodeId(0),
            role,
            size: 1,
            name: None,
            states: List::with_capacity(9),
            styles: List::with_capacity(limits.styles_per_node),
            text: List::with_capacity(runs),
            value: None,
            link: None,
            key: None,
            binding: None,
            slot: None,
            drawing: None,
        }
    }

    pub(crate) fn text_bytes(&self) -> u32 {
        let mut bytes = 0_u32;
        if let Some(name) = &self.name {
            bytes = length(name);
        }
        if let Some(value) = &self.value {
            bytes = bytes.checked_add(length(&value.text)).expect("node bytes fit u32");
        }
        for run in &self.text {
            bytes = bytes.checked_add(length(&run.text)).expect("node bytes fit u32");
        }
        bytes
    }
}

pub(crate) fn length(bytes: &[u8]) -> u32 {
    u32::try_from(bytes.len()).expect("view bytes fit u32")
}

/// A single tree in preorder, sized before building.
#[derive(Debug)]
pub struct Tree {
    pub(crate) nodes: List<Node>,
}

impl Tree {
    /// Reserve node storage once.
    #[must_use]
    pub fn new(capacity: u32) -> Tree {
        Tree { nodes: List::with_capacity(capacity) }
    }
    /// The tree's preorder nodes.
    #[must_use]
    pub fn nodes(&self) -> &[Node] {
        self.nodes.as_slice()
    }
    /// A node by its preorder index.
    #[must_use]
    pub fn get(&self, index: u32) -> Option<&Node> {
        self.nodes.get(index)
    }
    /// A node by its current visible identity.
    #[must_use]
    #[expect(clippy::manual_find, reason = "step code uses bounded scans without closures")]
    pub fn find(&self, id: NodeId) -> Option<&Node> {
        for node in &self.nodes {
            if node.id == id {
                return Some(node);
            }
        }
        None
    }
    /// Resolve an act's token in this displayed tree. A removed or unbound node
    /// gives nothing. A link's address binding also names its navigation act.
    #[must_use]
    pub fn bound(&self, id: NodeId) -> Option<crate::Binding> {
        if id == NodeId(0) {
            return None;
        }
        let node = self.find(id)?;
        node.binding.or(node.link)
    }

    /// Resolve the whole shell act; the owner interprets its token and value.
    #[must_use]
    pub fn act(&self, id: NodeId, act: crate::Act) -> Option<crate::BoundAct> {
        let binding = self.bound(id)?;
        Some(crate::BoundAct { binding, act })
    }

    /// Ask the view root to translate this link owner's address for the shell.
    #[must_use]
    pub fn address(&self, id: NodeId) -> Option<crate::Binding> {
        if id == NodeId(0) {
            return None;
        }
        self.find(id)?.link
    }

    /// The number of nodes, including every fallback child.
    #[must_use]
    pub fn len(&self) -> u32 {
        self.nodes.len()
    }
    /// Whether no nodes were built.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
