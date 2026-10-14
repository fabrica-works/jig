# Session 02: the page's outline and the frame's parts

Every page's tree gets one outline of landmarks: the banner first, with
jig's link, what needs the party and who is signed in; the navigation;
one main; then `after_page`'s parts. jig's frame, one region today,
becomes its parts, each a function of `jig-view` with its region name;
jig's pages, the test application's page and `ops`'s pages go inside a
main; and the referee checks the outline on every tree a person sees, in
jig's worlds and `ops`'s. The pages' insides stay as they are: jig 05
and 06 rebuild them from sections.

## Read first

- `client/domain/view.md`, 3.1 (the outline and its check), 4 (the view
  root's order), 4.1 ("The frame's parts", "The pages", `after_page`),
  11 ("Landmarks placed"), 12.
- `client/domain/pages.md`, 2 (the frame), 12.1, 12.3 ("What every
  arrangement keeps"), 12.4.
- `client/domain/testing.md`, 6 ("The page's outline").
- findings/view.md, 3, 4 and 6.

## What the design asks

- The page node's children: the banner, first, holding `jig-link`,
  `jig-needs` and `jig-party`; then navigation, main, complementary and
  content info landmarks, exactly one main and at most one content info;
  last, `after_page`'s owed cards (a complementary, `jig-owed`), the
  confirmation (a dialog, `jig-confirmation`) and the notices (a region,
  `jig-notices`), each when there is one. Nothing else; each shortcut at
  most once. `jig_view::outline::check(tree)` gives the first fault, or
  none (view.md, 3.1).
- The frame's parts `frame::link`, `needs`, `party`, `projects`,
  `go_to_task`, each a region of the banner with its region name;
  `frame::navigation`, a navigation landmark of jig's entries with the
  application's slot after them; `frame::banner`, the banner landmark
  with the five in that order (view.md, 4.1; pages.md, 12.1).
- `pages::main` builds the main with jig's page at the address, or
  nothing and false when the page is the application's;
  `pages::not_the_work` opens the main for a page not about the work
  (view.md, 4.1).
- The referee runs the check on every tree a person saw (testing.md, 6).

## Choices this plan makes

- The banner keeps the frame's name today, `JigPhrase::Frame`; the
  navigation landmark keeps `JigPhrase::Navigation`. Six phrases are
  added in 02.2, at the ids 674 to 679, in this order:
  `JigPhrase::NothingKnown` ("Nothing known yet"), which the needs' part
  shows before the frame's first delivery (pages.md, 12.1), and the
  names of the five parts' regions, `LinkPart` ("Connection"),
  `NeedsPart` ("What needs you"), `PartyPart` ("Who is signed in"),
  `ProjectsPart` ("Projects") and `GoToTaskPart` ("Go to a task"). The
  party's part shows `JigPhrase::SignedOut` before the first delivery,
  as the frame does today. 02.3 adds `JigPhrase::NoticesPart`
  ("Notices"), at the id 680. No phrase is removed.
- `Fault { rule: Rule, node: u32 }`, `node` the index in preorder of the
  node that breaks `rule` (the page node, index 0, when what is missing
  has no node). The check stops at the first fault in preorder.
- The world runs the check when the view emitted patches, beside
  `A::seen` (`tests/client/src/harness/world.rs:605-625`), and reports
  a fault as `Seen::Outline { fault }`; the referee fails on it
  (decisions.md, 7).
- `frame_link` (`tests/client/src/observe.rs:304-326`) finds the link's
  region by its region name, `JigRegion::Link.region()`: it is the
  observer's reading for the referee, not the person's query (the person
  never queries by region name, testing.md, 3).

## Rules that bind it

- The literal bounds of `crates/jig-view/src/tests/page_capacity.rs:845-853`
  (`each_page_bound_keeps_its_value_at_the_client_limits`) change in
  02.3 only by the nodes it adds, named in the commit body
  (code-organization.md, 4, "Literal bounds"); 02.2 changes none of
  them; no other expected value changes.
- Every `Role::Region` this session opens has a non-empty name:
  `crates/jig-view/src/tests/cards_access.rs:21-23` asserts it on the
  trees of jig-view's scenario tests. The frame's parts take the names
  of "Choices this plan makes"; in 02.3, a page not about the work names
  its region with its level-1 heading's phrase, and the notices' region
  takes `JigPhrase::NoticesPart`. A further region with no name takes a
  new phrase at the next free id, listed in the commit body.
- A story keeps its steps: only a scope (`within`) that named the old
  frame's regions changes, to the banner or the navigation landmark.
- Aim for the session's tests to add at most 0.3 s to the focused
  suite's median: a guide (README, "Test time"), checked by each
  landing's `Gate-Result`.

## Increments

### 02.1 The outline's check
Scope: view.md, 3.1.
Needs: jig 01.2 (the landmark roles) and 01.3 (`Region`, `JigRegion`,
  `Node::shortcut`).
Files: new `crates/jig-view/src/outline.rs`; `crates/jig-view/src/lib.rs`
  (`pub mod outline;`); new `crates/jig-view/src/tests/outline.rs`,
  registered in `crates/jig-view/src/tests.rs`.
Shape: `pub fn check(tree: &Tree) -> Option<Fault>`; `pub struct Fault {
  pub rule: Rule, pub node: u32 }`; `pub enum Rule` as in the sketch
  below. One pass over the page node's children (stepping by `size`),
  then one pass over the whole tree for shortcuts, keeping the shortcuts
  seen in a `List<Shortcut>` sized by the tree's length. A `Role::Main`,
  `Navigation`, `Complementary` or `ContentInfo` child after one of
  `after_page`'s parts is `AfterPageLast`; a complementary named
  `jig-owed`, a dialog named `jig-confirmation` and a region named
  `jig-notices` are `after_page`'s parts, in that order.
Test shape: `crates/jig-view/src/tests/outline.rs`, trees built by hand
  with `Builder::new(&super::tree::LIMITS)` (`tests/tree.rs:4`):
  `a_page_keeping_its_outline_has_no_fault`,
  `a_page_without_its_banner_first_is_a_fault`,
  `a_banner_without_the_party_region_is_a_fault`,
  `a_page_with_two_mains_is_a_fault`,
  `a_region_hanging_from_the_page_is_a_fault`,
  `a_main_after_the_notices_is_a_fault`,
  `a_shortcut_named_twice_is_a_fault_at_its_second_node`. Reference:
  ```rust
  #[test]
  fn a_page_with_two_mains_is_a_fault() {
      let mut b = page();
      banner(&mut b);
      b.open(Role::Main);
      b.close();
      b.open(Role::Main);
      b.close();
      assert_eq!(check(&finish(b)), Some(Fault { rule: Rule::OneMain, node: 6 }));
  }
  ```
  (`fn page() -> Builder` opens the page node; `fn finish(b: Builder) ->
  Tree` closes it and finishes the tree; `banner` builds the banner with
  the link, needs and party regions, one node each: the page is node 0,
  the banner and its regions nodes 1 to 4, the mains nodes 5 and 6. The
  three helpers go in this test file, and take no closure:
  `clippy.toml:63-65` disallows `Fn`, `FnMut` and `FnOnce`.) Each under
  20 ms.
Proof: unit; 02.3 runs it on jig's pages and 02.4 in the referee.
Don't: read names or words; read anything but roles, region names and
  shortcuts.
Size: at most 120 lines of production code.
Removes: nothing.

### 02.2 The frame's parts
Scope: view.md, 4.1 ("The frame's parts"); pages.md, 12.1.
Needs: 02.1 is not needed; jig 01.2 and 01.3.
Files: `crates/jig-view/src/frame.rs:13-96` (`frame`, split; `control`
  `:98-117` and `membership` `:119-143` stay); `crates/jig-view/src/pages.rs:123-132`
  (`jig`); `crates/jig-view/src/sizes.rs:189-199` (`frame_nodes`) and
  `crates/jig-view/src/lib.rs:52-56` (exports); `crates/jig-view/src/limits.rs:102-108`.
  The callers of `frame::frame` and `frame_nodes` (findings/view.md, 3):
  `crates/jig-view/src/tests/frame_facts.rs:9-27` and its phrase list
  `:186-199`, `tests/cards.rs:205`, `tests/cards_sizes.rs:379-388`,
  `tests/page_capacity.rs:518`, where `banner_nodes` plus
  `navigation_nodes` take `frame_nodes`' place and count 5 more: a
  banner, five regions and a navigation landmark replace two regions,
  and `Limits.nodes` grows by 5 (`:845-853` counts no frame, and stays).
  The observer: `tests/client/src/observe.rs:304-326`
  (`frame_link`) and `tests/client/src/observe/link_tests.rs:26-46`.
  Any story or test whose `within` names the region "Work" or
  "Navigation": `git grep -n -E 'b"Work"|b"Navigation"' -- tests testing
  examples crates` lists them (at the base, `observe.rs:309`,
  `link_tests.rs:34,37` and the phrases themselves,
  `crates/jig-view/src/phrases/english.rs:765-766`). The six new phrases:
  `crates/jig-view/src/phrases/jig.rs` (the enum, the ids 674 to 679
  beside `OwnerHoldRelease => 673` (`:1093`), their arms in `shape`
  (`:1723`) beside `SignedOut`'s (`:1936`), and `ALL`, `:2386`, 620
  entries becoming 626) and `crates/jig-view/src/phrases/english.rs`
  (beside `:765`).
Shape: in `frame.rs`, each `pub fn x(b: &mut Builder, client: &Domain,
  words: &Words, bindings: &mut JigBindings)` for `link`, `needs`,
  `party`, `projects`, `go_to_task`, each opening a `Role::Region` with
  `b.region(JigRegion::X.region())`, naming it with its phrase of
  "Choices this plan makes" (`b.name(&words.say(JigPhrase::LinkPart,
  &[]))`), and building what findings/view.md, 3 lists for it; a
  parameter a part does not read keeps its place in view.md 4.1's
  signature, named with a leading underscore (`_bindings` in `link`,
  `_client` in `go_to_task`: `scripts/check.sh` denies warnings);
  without a frame's document, `needs` shows
  `JigPhrase::NothingKnown` and `party` shows `JigPhrase::SignedOut`
  (`frame.rs:51` today);
  `pub fn navigation(b, client, nav: SlotRef, words, bindings)`, a
  `Role::Navigation` with `JigRegion::Navigation`; `pub fn banner(b,
  client, words, bindings)`, a `Role::Banner` holding the five in
  pages.md 12.1's order. `jig()` calls `banner` then `navigation`.
  `frame::frame` goes. Companions in `sizes.rs`: `link_nodes`,
  `needs_nodes`, `party_nodes`, `projects_nodes`, `go_to_task_nodes`,
  `navigation_nodes`, `banner_nodes` (the sum of the five and 1), each
  `pub fn x_nodes(limits: &jig_domain_client::Limits) -> Option<u32>`,
  replacing `frame_nodes`; `Limits` adds `banner_nodes` and
  `navigation_nodes` where it added `frame_nodes`.
Test shape: `crates/jig-view/src/tests/frame_facts.rs`: its tests build
  `frame::banner` and `frame::navigation` where they built
  `frame::frame`; add `the_banner_holds_link_needs_party_projects_and_task_in_order`
  (the region names of the banner's children, as literals) and
  `the_navigation_landmark_ends_with_the_applications_slot` and
  `the_needs_part_says_nothing_is_known_before_the_frame`. Each under
  20 ms.
Proof: world, `heavy cargo nextest run -p jig-client-world -p jig-ops-world`
  once: the stories that read the link's state (`tests/client/tests/shell.rs`,
  the fault stories) pass with the link in the banner, and every story
  that presses "Sign out", follows a needs link or goes to a task by
  number passes unchanged.
Don't: change a phrase, the order of jig's navigation entries, or what a
  part shows.
Size: at most 120 lines of production code.
Removes: `frame::frame`, `frame_nodes`.

### 02.3 The main, and `after_page`'s parts
Scope: view.md, 3.1, 4 (the view root's order) and 4.1 ("The pages",
  `after_page`); pages.md, 12.1 ("After the page"), 12.2's pages not
  about the work, 12.3.
Needs: 02.1 and 02.2.
Files: `crates/jig-view/src/pages.rs:103-133` (`page` becomes `main`;
  `jig`), `:25` (`heading`, public), `:174-210` (`after_page`); the
  callers of `pages::page` and `pages::not_the_work`:
  `crates/jig-view/src/tests/history_pages.rs:123`,
  `crates/jig-view/src/tests/pages.rs:26-31`;
  `crates/jig-view/src/pages/not_the_work.rs`,
  `crates/jig-view/src/confirmation.rs:20` (the dialog's region name),
  `crates/jig-view/src/notices.rs:16-27` (the notices inside a region,
  named `JigPhrase::NoticesPart`: `crates/jig-view/src/phrases/jig.rs`,
  the id 680, `ALL`'s 626 entries becoming 627, and
  `crates/jig-view/src/phrases/english.rs`);
  the counts: the main is counted once in `Limits` (`limits.rs:102-108`,
  beside the page node's `1`: 2 nodes), and so is the owed cards'
  aside (1 more); `notices_nodes` gains its region (1); the page
  companions do not change but `not_the_work_nodes` (its region: 1, so
  the literal at `crates/jig-view/src/tests/page_capacity.rs:852`, 28,
  becomes 29; the other literals of `:845-853` stay); the roots:
  `testing/jig-test-view/src/root.rs:57-61` and
  `examples/ops/view/src/root.rs:90-104` (the application's page inside
  a `Role::Main`); `crates/jig-view/src/tests/page_capacity.rs:550-552`
  (a test page built the same way); a new test in
  `crates/jig-view/src/tests/pages.rs`.
Shape: `pub fn main(b, client, words, bindings) -> bool`: at a jig
  address, opens `Role::Main`, builds the page as `page` does today,
  closes it, returns true; on a page not about the work, calls
  `not_the_work`; at `Page::Elsewhere`, builds nothing and returns
  false. `pub fn not_the_work(b, client, words, bindings) -> bool`: on
  a page not about the work, opens the main, builds it inside a region
  with its region name (`jig-starting`, `jig-signing-in`, `jig-missing`,
  `jig-out-of-date`, `jig-someone-else`), returns true; otherwise builds
  nothing and returns false. `pub fn heading(b, words, phrase, holes)`.
  `after_page`: the owed cards inside one `Role::Complementary` with
  `JigRegion::Owed`, opened only when there is at least one; the
  confirmation's dialog carries `JigRegion::Confirmation`; the notices
  and their dismiss button inside one `Role::Region` with
  `JigRegion::Notices`, opened only when there is a notice. `jig()`:
  banner, navigation, then `main`; when it returns true, `after_page`.
  The roots: when `jig_view::jig` returns false, `b.open(Role::Main)`,
  the family's page, `b.close()`, then `after_page`, as today.
Test shape: `crates/jig-view/src/tests/pages.rs`:
  `each_of_jigs_pages_keeps_the_outline`, a table test whose rows are
  domain states the file's helpers already reach (`start`, `drive` and
  `document` of `tests/cards.rs`, `deliver` of `tests/pages.rs:36`;
  `special_pages`' states at
  `tests/pages.rs:42`): the inbox with its frame, conversations, a
  task's overview, a project's overview, the deployment, "page missing"
  and "signing in"; each row builds the whole tree with `crate::jig`
  under the page node (a `SlotRef` of `Owner::Application { number: 1 }`
  for the navigation's slot), asserts `outline::check(&tree) == None`,
  and is labelled with its page. Then
  `the_owed_cards_sit_in_their_aside_after_the_main`. Each row under
  20 ms. The client's world checks every other page a story reaches
  (02.4).
Proof: world, the command of 02.2's Proof, once: every story of the
  client's and `ops`'s worlds passes with jig's pages, the test
  family's and `ops`'s inside a main.
Don't: change what a page builds inside its main; decide the owed cards
  otherwise than today (jig 06 does it from the domain).
Size: at most 100 lines of production code.
Removes: `pages::page` (become `main`).

### 02.4 The referee checks the outline
Scope: testing.md, 6 ("The page's outline"); view.md, 3.1 ("Checked").
Needs: 02.3.
Files: `tests/client/src/harness/world.rs:605-625` (the check beside
  `A::seen`), `tests/client/src/harness/observation.rs:23-32` (`Seen`),
  `tests/client/src/harness/referee.rs:220-292` (`shown`),
  `tests/client/tests/referee.rs` (a test).
Shape: `Seen::Outline { fault: jig_view::outline::Fault }`; in the
  world, when the view emitted patches: `if let Some(fault) =
  jig_view::outline::check(A::tree(&c.view)) { seen.push(Seen::Outline
  { fault }) }`; in `shown`, `Seen::Outline { .. } => judge.check(false,
  "the page's outline: a tree broke view.md 3.1")`.
Test shape: `tests/client/tests/referee.rs`, beside the test at `:424`
  that feeds `Observation::Shown` by hand:
  `a_tree_breaking_the_outline_fails_the_referee`. Under 0.5 s.
Proof: world and joint, `heavy cargo nextest run -p jig-client-world -p
  jig-ops-world` once: the client's world, the joint world and `ops`'s
  client world pass with the check on every changed tree; the report
  names the focused suite's time from the landing's `Gate-Result`.
Don't: run the check on a tree that did not change; format the tree on
  a fault (the fault's rule and node are the message).
Size: at most 30 lines (test kit code).
Removes: nothing.

## Interface sketches

```rust
// crates/jig-view/src/outline.rs
/// The first node, in preorder, that breaks view.md 3.1, and the rule it breaks.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Fault { pub rule: Rule, pub node: u32 }
/// One rule of the page's outline.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rule {
    /// The page node's first child is not the banner.
    BannerFirst,
    /// The banner holds no region of this name: jig-link, jig-needs or jig-party.
    BannerHolds { region: JigRegion },
    /// A second main, or none (then `node` is the page).
    OneMain,
    /// A second content info.
    OneContentInfo,
    /// A child of the page that is no landmark and none of `after_page`'s parts.
    InsideLandmark,
    /// A landmark after one of `after_page`'s parts, or those parts out of order.
    AfterPageLast,
    /// A shortcut that a node before it names too.
    ShortcutOnce,
}
pub fn check(tree: &Tree) -> Option<Fault>;

// crates/jig-view/src/frame.rs
pub fn link(b: &mut Builder, client: &Domain, words: &Words, _bindings: &mut JigBindings);
pub fn needs(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings);
pub fn party(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings);
pub fn projects(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings);
pub fn go_to_task(b: &mut Builder, _client: &Domain, words: &Words, bindings: &mut JigBindings);
pub fn navigation(b: &mut Builder, client: &Domain, nav: SlotRef, words: &Words, bindings: &mut JigBindings);
pub fn banner(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings);

// crates/jig-view/src/pages.rs
pub fn heading(b: &mut Builder, words: &Words, phrase: JigPhrase, holes: &[Hole]);
pub fn main(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) -> bool;
pub fn not_the_work(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) -> bool;
pub fn after_page(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings);
pub fn jig(b: &mut Builder, client: &Domain, nav: SlotRef, words: &Words, bindings: &mut JigBindings) -> bool;
```

Who owns what: this session owns the outline's check, the frame's
parts and `pages::{heading, main, not_the_work}`; jig 05 and 06 change
what `main` builds inside the landmark, never its signature.

## Questions to raise when reached

None.

## Done when

- `heavy cargo nextest run -p jig-view -E 'test(each_of_jigs_pages_keeps_the_outline)'`
  passes on `main`, once.
- `git grep -n 'outline::check' origin/main -- tests/client/src/harness`
  prints the world's line.
