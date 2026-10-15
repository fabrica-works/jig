use crate::ui::Limits;

pub(crate) const LIMITS: Limits = Limits {
    nodes: 12,
    depth: 4,
    text_per_node: 32,
    text_bytes: 128,
    drawing_bytes: 8,
    styles_per_node: 2,
    patches: 72,
    bindings: 4,
    slots: 4,
    markdown_depth: 2,
};
