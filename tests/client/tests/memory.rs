use jig_client_world::{Settings, stories};
#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

/// A real client allocation crosses a deliberately small declared budget.
/// The same callback guard used by the full fixture must catch it.
#[test]
#[should_panic(expected = "bounded: client and view exceeded the heap envelope")]
fn a_client_that_exceeds_its_declared_heap_budget_is_caught() {
    let mut world = stories::configured(Settings { memory: true, heap_bytes_per_client: 1024, ..Settings::default() });
    world.client(7, stories::inbox());
}
