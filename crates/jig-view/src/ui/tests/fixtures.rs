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

/// Start a generic page with a single-node banner.
pub(crate) fn page() -> crate::ui::Builder {
    let mut builder = crate::ui::Builder::new(&LIMITS);
    builder.open(crate::ui::Role::Page);
    builder.open(crate::ui::Role::Banner);
    builder.close();
    builder
}

pub(crate) fn finish(mut builder: crate::ui::Builder) -> crate::ui::Tree {
    builder.close();
    builder.finish()
}
