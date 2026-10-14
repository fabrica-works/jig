# Session 01: the tree's closed sets

The view's tree completes its closed sets before anything builds with
them: owners become jig or an application's number; the landmarks, the
menu, the disclosure and the image join the roles; a region name, a
shortcut and a picture join a node's attributes, with jig's region names
as one closed set; and the act on a drawing gets its place among the
shell's acts. Each lands with every exhaustive match it breaks, in
`jig-view`, the fake person, jig's test view, `ops`'s view and the
worlds. Nothing builds a landmark or a region name yet: jig 02 does.

## Read first

- `client/domain/view.md`, section 3 (the tree, "Owners", menus,
  disclosures, images, "Closed"), section 5 (the act on a drawing, the
  shortcut), section 10 (a drawing's bytes) and section 13 ("Region names
  are not styles").
- `client/domain/root.md`, section 15 (owners).
- `client/domain/pages.md`, 12.1, 12.2 and 12.4 (jig's region names).
- decisions.md, 3 and "For the user".

## What the design asks

- `Owner` is `Jig`, or `Application { number }`; there is no other kind
  (view.md, 3; root.md, 15).
- The roles add the landmarks banner, navigation, main, complementary,
  content info, form and search; menu and menu item; disclosure; image
  (view.md, 3).
- A node may carry a region name (an owner and a number from its closed
  set), a shortcut (`Shortcut { key, modifier }`: a letter `a`–`z` or a
  digit `0`–`9`, with no modifier, the platform's primary one, or Alt),
  and, on an image, a picture (an owner and a number) (view.md, 3 and 5).
- jig's region names are `jig_view::regions::JigRegion`, each with its
  name (pages.md, 12.4), and are those pages.md, 12.1 and 12.2 list.
- `Act::Drawing { data }` beside `Press`, `Input`, `Submit` and `Toggle`;
  every owner's view matches it, jig's decodes it into nothing; past the
  view's limit on a drawing's bytes the view drops it (view.md, 5).

## Choices this plan makes

- `Region { owner, number: u16 }`, `Picture { owner, number: u16 }`,
  `Shortcut { key: u8, modifier: Modifier }` with `Modifier::{Plain,
  Primary, Alt}` (decisions.md, "For the user").
- `JigRegion`'s numbers are its declaration order, in pages.md 12.1's
  order then 12.2's (the table, then the regions named inside sections,
  then the pages not about the work), from 0.
- The diff carries the new attributes in its `Attribute` (one patch
  replaces a node's attributes, as now): no new kind of patch, and the
  patch bound is unchanged.
- The fake person presses a menu item as it presses a button; it acts
  on no landmark, disclosure or image.

## Rules that bind it

- "No type beside an existing one for the same thing: change the type in
  place, and its users in the same increment" (code-organization.md, 5):
  `Owner::Side` and `Owner::Other` go in 01.1, with every user.
- No `_ =>` arm in a match on `Role`, `Owner` or `Act`
  (code-organization.md, 5); clippy's `wildcard_enum_match_arm` denies
  it in every crate that has the lint.
- Aim for the session's tests to add at most 0.2 s to the focused
  suite's median: a guide (README, "Test time"), checked by each
  landing's `Gate-Result`.

## Increments

### 01.1 Owners are jig, or an application's number
Scope: view.md, 3 ("Owners"); root.md, 15 ("Matched whole").
Needs: nothing.
Files: `crates/jig-view/src/tree.rs:13-22` (`Owner`). Every user of
  `Owner::Side` or `Owner::Other`, by `git grep -c -E
  'Owner::(Side|Other)\b' -- '*.rs'` at the base (37 files, 66 uses):
  `crates/jig-view/src/{cards/common.rs,proposed_task.rs}` and its tests
  (`tests/{binding,cards,cards_objects,cards_sizes,diff,frame_facts,owner_deployment,page_capacity,phrases,proposed_task,slots,tree,turns}.rs`);
  `examples/ops/view/src/{bindings,drawing,infrastructure,observability,root,tests,words}.rs`;
  `testing/jig-fake-person/src/tests.rs`;
  `testing/jig-test-view/src/{bindings,connector_values_words,drawing,family,root,words}.rs`
  and `src/tests/{connector_values,family_terminals,world_limits}.rs`;
  `tests/client/src/{harness/observation.rs,joint/deployment_story.rs,observe.rs}`;
  `tests/ops/src/client/{app,stories}.rs`. The exhaustive matches on
  `Owner` among them (findings/view.md, 1):
  `testing/jig-test-view/src/root.rs:66,183`,
  `examples/ops/view/src/root.rs:109,205`, `tests/client/src/observe.rs:267`,
  `tests/client/src/harness/observation.rs:102`,
  `tests/client/src/joint/deployment_story.rs:105`. Doc comments that call
  an owner "a connector's client side" where they mean any owner
  (`tree.rs:18`).
Shape: `pub enum Owner { Jig, Application { number: u16 } }`, one doc line
  each (jig's own view; one of the application's owners, by the number
  both its roots share). `Owner::Side { connector: n }` becomes
  `Owner::Application { number: n }`; each root's match becomes
  `Owner::Application { number } => match Connector::of(number) { … }`
  (`testing/jig-test-domain-client/src/owner.rs:19`; for `ops`,
  `jig_ops_domain_connectors::connector(number)`,
  `examples/ops/domain-connectors/src/lib.rs:28`), and the
  `Owner::Other { .. }` arms go (`testing/jig-test-view/src/root.rs:136,195`,
  `examples/ops/view/src/root.rs:181,215`). A number no owner has is the arm that today handles
  `Connector::of` giving `None`.
Test shape: in `testing/jig-test-view/src/tests.rs`, one test,
  `a_slot_of_a_number_no_owner_has_is_unshowable`: a tree whose slot's
  reference is `Owner::Application { number: 9 }` (no connector of the
  test application has 9) is filled with the test view's unshowable
  words. Every other test changes only its owner's spelling.
Proof: world, `heavy cargo nextest run -p jig-client-world -p jig-ops-world`
  once: the client's and `ops`'s worlds pass, every slot of theirs still
  filled by its owner (their stories and referee). About 2 minutes
  with the build (an estimate, not a bound).
Don't: rename `Part`'s field `connector`, `jig-ops-domain-connectors`, or
  the client link's `family` (decisions.md, 5); keep a `Side` alias.
Size: at most 80 lines of production code (most of the change is
  renames).
Removes: `Owner::Side`, `Owner::Other`.

### 01.2 Landmarks, menus, disclosures and images among the roles
Scope: view.md, 3 (the roles; "A menu"; "A disclosure"; "An image").
Needs: nothing.
Files: `crates/jig-view/src/tree.rs:24-98` (`Role`); the three
  exhaustive matches on `Role` (findings/view.md, 2):
  `testing/jig-fake-person/src/tree.rs:117` (the role's words),
  `testing/jig-fake-person/src/random.rs:80,118`; `Face::allows`
  (`testing/jig-fake-person/src/tree.rs:98-112`); the role lists a menu
  item joins, `crates/jig-view/src/tests/cards_access.rs:14` and
  `crates/jig-view/src/tests/binding.rs:10`;
  `testing/jig-fake-person/src/tests.rs`.
Shape: `Role` gains, after `Page`, `Banner`, `Navigation`, `Main`,
  `Complementary`, `ContentInfo`, `Form`, `Search`, and after `Separator`,
  `Menu`, `MenuItem`, `Disclosure`, `Image`, one doc line each. The fake
  person's words for them: "banner", "navigation", "main",
  "complementary", "content info", "form", "search", "menu", "menu
  item", "disclosure", "image". Its `allows` takes `Act::Press` on a
  menu item; its random person presses a menu item as a button, and acts
  on none of the others.
Test shape: `testing/jig-fake-person/src/tests.rs`, each its own
  `#[test]`, with the file's `build` (`:43`) and `binding` (`:51`)
  helpers: `a_menu_item_inside_a_named_menu_is_found_and_pressable`,
  `a_disclosures_content_is_found_by_its_text`, and
  `the_random_person_never_acts_on_a_landmark`. Reference:
  ```rust
  #[test]
  fn a_menu_item_inside_a_named_menu_is_found_and_pressable() {
      let view = build(|b| {
          b.open(Role::Page);
          b.open(Role::Menu);
          b.name(b"More");
          b.open(Role::MenuItem);
          b.name(b"Archive");
          b.bind(binding(1));
          b.close();
          b.close();
          b.close();
      });
      let mut face = TreeFace::new(view.tree());
      let menu = Target::Role { role: Role::Menu, name: Box::from(*b"More") };
      let Found::Node(item) = face.find(&Find::named(Role::MenuItem, b"Archive").within(Box::from([menu]))) else {
          panic!("the menu item is found inside its menu");
      };
      assert!(face.allows(item, &Act::Press));
  }
  ```
  Each under 20 ms.
Proof: unit; nothing builds these roles until jig 02.2 (the banner and
  the navigation) and 02.3 (the main), which prove them in the client's
  world.
Don't: give a menu item an act of its own; give the fake person a query
  by region name (testing.md, 3: "never … by region name").
Size: at most 60 lines of production code.
Removes: nothing.

### 01.3 Region names, shortcuts and pictures among a node's attributes
Scope: view.md, 3 (a node's attributes), 5 ("A shortcut"), 13; pages.md,
  12.4.
Needs: nothing (01.1 if it has landed: `Owner` in the new structs).
Files: `crates/jig-view/src/tree.rs` (`Node`, `Node::vacant`,
  `Node::new`), `crates/jig-view/src/builder.rs` (new methods beside
  `style` and `key`, `:163-178`), `crates/jig-view/src/diff.rs:14-41`
  (`Attribute`, `Attribute::of`), `crates/jig-view/src/lib.rs:57`
  (exports), new `crates/jig-view/src/regions.rs`; tests in
  `crates/jig-view/src/tests/{tree,diff}.rs` and a new
  `crates/jig-view/src/tests/regions.rs` (registered in
  `crates/jig-view/src/tests.rs`). Every struct literal of `Node` or
  `Attribute` the compiler names.
Shape:
  - `tree.rs`: `pub struct Region { pub owner: Owner, pub number: u16 }`;
    `pub struct Picture { pub owner: Owner, pub number: u16 }`;
    `pub struct Shortcut { pub key: u8, pub modifier: Modifier }`;
    `pub enum Modifier { Plain, Primary, Alt }`; `Node` gains `region:
    Option<Region>`, `shortcut: Option<Shortcut>`, `picture:
    Option<Picture>`.
  - `builder.rs`: `pub fn region(&mut self, region: Region)` (asserts the
    open node is a landmark, a region, a dialog, tabs, or a log: pages.md
    12.1 and 12.2 name the confirmation's dialog, the tabs and the
    conversation's log, decisions.md, 13); `pub fn shortcut(&mut self,
    shortcut: Shortcut)` (asserts the key is `b'a'..=b'z'` or
    `b'0'..=b'9'` and the node is bound: call it after `bind`); `pub fn
    picture(&mut self, picture: Picture)` (asserts the node is an image).
  - `diff.rs`: `Attribute` gains `region`, `shortcut` and `picture`,
    copied by `Attribute::of`; a matched node whose one of them changed
    gets one `Patch::Attribute`, as a changed style does today.
  - `regions.rs`: `pub enum JigRegion` with the 48 names of pages.md
    12.1 and 12.2 (interface sketch below), `pub const fn name(self) ->
    &'static [u8]`, `pub const fn region(self) -> Region` (owner `Jig`,
    its number), `pub fn of(number: u16) -> Option<JigRegion>`.
Test shape: `crates/jig-view/src/tests/regions.rs`:
  `every_jig_region_has_its_designed_name`, one table test whose rows are
  `(JigRegion::Link, b"jig-link")` and so on, every variant, the names
  copied from pages.md 12.1 and 12.2 as literals;
  `a_region_name_survives_its_number` (`JigRegion::of(r.region().number)
  == Some(r)` for three variants, written out). In `tests/diff.rs`,
  `a_changed_region_name_is_one_attribute_patch`. In `tests/tree.rs`,
  `a_shortcut_on_a_key_outside_letters_and_digits_is_refused`
  (`#[should_panic]`, key `b'!'`). Build with `Builder::new(&LIMITS)`
  (`tests/tree.rs:4`); each under 20 ms.
Proof: unit; jig 02.2 puts the first region names on jig's frame and
  proves them in the client's world.
Don't: print, style or place anything by region name in jig-view; give a
  region name to a node that is no landmark, region, dialog, tabs or log.
Size: at most 200 lines of production code (`regions.rs`'s table is
  most of it).
Removes: nothing.

### 01.4 The act on a drawing
Scope: view.md, 5 ("The act on a drawing"); 10 (its bytes).
Needs: nothing.
Files: `crates/jig-view/src/binding.rs:15-26` (`Act`); the 17
  exhaustive matches on `Act` (findings/view.md, 2):
  `crates/jig-view/src/binding.rs:233,243,253,257,261,274,280,286,290,298`,
  `testing/jig-test-view/src/bindings.rs:55,59`,
  `examples/ops/view/src/bindings.rs:59,63,76`,
  `testing/jig-fake-person/src/tree.rs:108`,
  `testing/jig-fake-person/src/tests.rs:331`; `crates/jig-view/src/tree.rs:315`
  (`Tree::act`), `crates/jig-view/src/render.rs:75` (`View::act`, where
  the limit is known); `testing/jig-fake-person/src/random.rs:50-60`
  (the random acts, which never make it).
Shape: `Act::Drawing { data: Box<[u8]> }`, doc line "bytes of the
  drawing's owner, reported by its drawing code in a shell". The view's
  `act` returns `None` for `Act::Drawing` whose `data` is longer than
  `limits.drawing_bytes`. jig's `decode` maps it to `None` in each of its
  arms; the test view's and `ops`'s decoders likewise. The fake person
  allows it on a `Role::Drawing` node only (`Face::allows`,
  `tree.rs:98`) and never makes one.
Test shape: `crates/jig-view/src/tests/binding.rs`:
  `an_act_on_a_drawing_past_its_bytes_is_dropped`,
  `jigs_view_decodes_an_act_on_a_drawing_into_nothing`. Each under 20 ms.
Proof: unit; no drawing is bound until drawings acted on (later.md), so
  nothing reports the act; the decoders' arms are the compiler's.
Don't: bind a drawing; decode the bytes anywhere.
Size: at most 40 lines of production code.
Removes: nothing.

## Interface sketches

```rust
// crates/jig-view/src/tree.rs
/// The view child that interprets a token, key, style, region, picture or phrase.
pub enum Owner {
    /// jig's own view.
    Jig,
    /// One of the application's owners, by the number both its roots share.
    Application { number: u16 },
}
/// A region's language-free name: its owner and a number of its closed set.
pub struct Region { pub owner: Owner, pub number: u16 }
/// One of an owner's pictures, which a shell takes when it is built.
pub struct Picture { pub owner: Owner, pub number: u16 }
/// A key a bound node answers to, as a press.
pub struct Shortcut { pub key: u8, pub modifier: Modifier }
/// The modifier a shortcut's key is pressed with.
pub enum Modifier { Plain, Primary, Alt }

// crates/jig-view/src/regions.rs
/// jig's region names (pages.md, 12.1 and 12.2), numbered in this order.
pub enum JigRegion {
    Link, Needs, Party, Projects, GoToTask, Navigation, Owed, Confirmation, Notices,
    InboxHeader, InboxNeeds, InboxUpdates, Conversations,
    TaskHeader, TaskTabs, TaskOverview, TaskActivity, TaskPlan, TaskHistory, TaskBudget, TaskPanels,
    ProjectHeader, ProjectTabs, ProjectOverview, ProjectGoals, ProjectTasks, ProjectNotes,
    ProjectSettings, ProjectResources, ProjectPolicy, ProjectRoles, ProjectSpend, ProjectRecurring,
    Deployment,
    Conversation, Composer, NewChat, NewGoal, NotesSearch, Hosts, Accounts, Connectors, Capacity,
    Starting, SigningIn, Missing, OutOfDate, SomeoneElse,
}
impl JigRegion {
    pub const fn name(self) -> &'static [u8];      // b"jig-link", …
    pub const fn region(self) -> Region;            // Region { owner: Owner::Jig, number }
    pub fn of(number: u16) -> Option<JigRegion>;
}

// crates/jig-view/src/binding.rs
pub enum Act {
    Press,
    Input { value: Box<[u8]> },
    Submit,
    Toggle,
    /// Bytes of the drawing's owner, reported by its drawing code in a shell.
    Drawing { data: Box<[u8]> },
}
```

Who owns what: this session owns `Owner`, `Region`, `Picture`,
`Shortcut`, `Modifier`, `JigRegion` and `Act::Drawing`; every later
session uses them by name.

## Questions to raise when reached

None: the design settles each.

## Done when

- `git grep -n -E 'Owner::(Side|Other)\b' origin/main -- '*.rs'` prints
  nothing.
- `heavy cargo nextest run -p jig-view -p jig-fake-person -p jig-test-view`
  passes on `main`, once.
