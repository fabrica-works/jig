# jig: the client audit's tried changes

Evidence for the plan's sessions, against jig `d91a89c45`: what the audit measured, not code to apply as it is. An increment's Shape is the session file's.

```diff
diff --git a/crates/jig-view/src/phrases.rs b/crates/jig-view/src/phrases.rs
index 0731f45df..f4d7c340c 100644
--- a/crates/jig-view/src/phrases.rs
+++ b/crates/jig-view/src/phrases.rs
@@ -231,7 +231,7 @@ impl Phrasebook {
         }
         // Id is the first derived ordering field. Equal ids become adjacent,
         // regardless of the caller's order or the rest of their declarations.
-        shapes.sort_unstable();
+        shapes.sort();
         for pair in shapes.windows(2) {
             let earlier = pair.first().expect("window has two shapes");
             let later = pair.last().expect("window has two shapes");
@@ -264,7 +264,7 @@ impl Phrasebook {
             if language.phrases.len() > usize::try_from(limits.phrases).expect("phrase count fits usize") {
                 return Err(Invalid::Limit);
             }
-            language.phrases.sort_unstable();
+            language.phrases.sort();
             for pair in language.phrases.windows(2) {
                 let earlier = pair.first().expect("window has two phrases");
                 let later = pair.last().expect("window has two phrases");
@@ -284,9 +284,14 @@ impl Phrasebook {
                 }
                 return Err(Invalid::Limit);
             }
-            for words in &language.phrases {
-                let Ok(index) = ids.binary_search(&words.id) else {
-                    return Err(Invalid::Unknown { id: words.id });
+            for (position, words) in language.phrases.iter().enumerate() {
+                let index = if ids.get(position) == Some(&words.id) {
+                    position
+                } else {
+                    let Ok(index) = ids.binary_search(&words.id) else {
+                        return Err(Invalid::Unknown { id: words.id });
+                    };
+                    index
                 };
                 let shape = shapes.get(index).expect("id table has the shapes' order");
                 check_forms(shape, words)?;
diff --git a/crates/jig-view/src/sizes/pages.rs b/crates/jig-view/src/sizes/pages.rs
index 369de3a54..041c1bae2 100644
--- a/crates/jig-view/src/sizes/pages.rs
+++ b/crates/jig-view/src/sizes/pages.rs
@@ -3,14 +3,24 @@
 //! is the sum of the owners' fragment bounds for the page, including navigation.
 use super::{authority, card_nodes, chip_nodes, markdown};
 use jig_domain_client::Limits;
-fn runtime(l: &Limits) -> Option<u32> {
-    chip_nodes(l)?.checked_mul(l.rows)?.checked_add(l.rows.checked_mul(5)?)?.checked_add(24)
+#[derive(Clone, Copy)]
+struct Shared {
+    card: u32,
+    chip: u32,
+}
+impl Shared {
+    fn of(l: &Limits) -> Option<Self> {
+        Some(Self { card: card_nodes(l)?, chip: chip_nodes(l)? })
+    }
+}
+fn runtime_in(l: &Limits, c: Shared) -> Option<u32> {
+    c.chip.checked_mul(l.rows)?.checked_add(l.rows.checked_mul(5)?)?.checked_add(24)
 }
 /// Inbox cards, lineage, filters, counts and both edges.
 #[must_use]
-pub fn inbox_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
-    card_nodes(l)?
-        .checked_add(chip_nodes(l)?)?
+fn inbox_nodes_in(l: &Limits, c: Shared, slot_nodes: u32) -> Option<u32> {
+    c.card
+        .checked_add(c.chip)?
         .checked_add(28)?
         .checked_mul(l.rows.checked_mul(4)?)?
         .checked_add(l.projects.checked_mul(3)?)?
@@ -19,22 +29,22 @@ pub fn inbox_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
 }
 /// Chat rows, every runtime alternative and all offered project forms.
 #[must_use]
-pub fn conversations_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
-    let row = card_nodes(l)?
-        .checked_add(chip_nodes(l)?)?
-        .checked_add(runtime(l)?)?
+fn conversations_nodes_in(l: &Limits, c: Shared, slot_nodes: u32) -> Option<u32> {
+    let row = c.card
+        .checked_add(c.chip)?
+        .checked_add(runtime_in(l, c)?)?
         .checked_add(crate::owner_budget::budget_nodes())?
         .checked_add(20)?
         .checked_mul(l.rows)?;
     let forms = l.charters.max(l.rows).checked_mul(3)?.checked_add(9)?.checked_mul(l.projects.max(l.rows))?;
     row.checked_add(forms)?.checked_add(66)?.checked_add(slot_nodes)
 }
-fn call(l: &Limits) -> Option<u32> {
-    card_nodes(l)?.checked_mul(l.rows)?.checked_add(l.rows.checked_mul(5)?)?.checked_add(16)
+fn call_in(l: &Limits, c: Shared) -> Option<u32> {
+    c.card.checked_mul(l.rows)?.checked_add(l.rows.checked_mul(5)?)?.checked_add(16)
 }
-fn turn(l: &Limits) -> Option<u32> {
+fn turn_in(l: &Limits, c: Shared) -> Option<u32> {
     // Every message has a full block list; a call can name a full object list.
-    call(l)?
+    call_in(l, c)?
         .checked_add(markdown(l)?)?
         .checked_add(6)?
         .checked_mul(l.rows)?
@@ -44,27 +54,27 @@ fn turn(l: &Limits) -> Option<u32> {
 }
 /// Committed log, pending words, all provisional turns, one independent current call and both edges.
 #[must_use]
-pub fn conversation_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
+fn conversation_nodes_in(l: &Limits, c: Shared, slot_nodes: u32) -> Option<u32> {
     let owner = 100_u32.checked_add(l.rows.checked_mul(4)?)?;
-    let rows = turn(l)?
+    let rows = turn_in(l, c)?
         .checked_add(owner)?
-        .checked_add(card_nodes(l)?.checked_mul(2)?)?
+        .checked_add(c.card.checked_mul(2)?)?
         .checked_add(20)?
         .checked_mul(l.rows)?;
-    let live = call(l)?.max(10).checked_add(markdown(l)?)?.checked_add(8)?.checked_mul(l.rows)?;
+    let live = call_in(l, c)?.max(10).checked_add(markdown(l)?)?.checked_add(8)?.checked_mul(l.rows)?;
     rows.checked_add(live)?.checked_add(l.conversation_words.checked_mul(9)?)?.checked_add(34)?.checked_add(slot_nodes)
 }
-fn plan(l: &Limits) -> Option<u32> {
-    let older = card_nodes(l)?
+fn plan_in(l: &Limits, c: Shared) -> Option<u32> {
+    let older = c.card
         .checked_mul(l.rows.checked_add(1)?)?
-        .checked_add(chip_nodes(l)?)?
+        .checked_add(c.chip)?
         .checked_add(l.rows.checked_mul(2)?)?
         .checked_add(18)?
         .checked_mul(l.rows)?
         .checked_add(26)?;
     older.max(crate::owner_tree::max_nodes(l)?.checked_add(6)?).checked_add(crate::historical_pages::ended_nodes(l)?)
 }
-fn overview(l: &Limits) -> Option<u32> {
+fn overview_in(l: &Limits, c: Shared) -> Option<u32> {
     crate::owner_chip::max_nodes(l)?
         .checked_add(l.rows.checked_add(20)?.checked_mul(l.rows)?)?
         .checked_add(crate::task_result::nodes(l)?)?
@@ -73,26 +83,26 @@ fn overview(l: &Limits) -> Option<u32> {
         .checked_add(crate::proposed_task::contract_nodes(l)?)?
         .checked_add(crate::owner_wake::policy_nodes())?
         .checked_add(crate::owner_wake::subscription_nodes(l.rows)?)?
-        .checked_add(chip_nodes(l)?.checked_add(4)?.checked_mul(l.rows)?.checked_mul(3)?)?
-        .checked_add(card_nodes(l)?.checked_mul(l.rows.checked_add(2)?)?)?
+        .checked_add(c.chip.checked_add(4)?.checked_mul(l.rows)?.checked_mul(3)?)?
+        .checked_add(c.card.checked_mul(l.rows.checked_add(2)?)?)?
         .checked_add(l.rows.checked_mul(36)?)?
         .checked_add(24)
 }
-fn history(l: &Limits) -> Option<u32> {
+fn history_in(l: &Limits) -> Option<u32> {
     let line = markdown(l)?.checked_mul(2)?.checked_add(authority(l)?)?.checked_add(24)?;
     let row = line.max(crate::owner_history::max_nodes(l)?).max(crate::owner_milestone::max_nodes(l)?);
     row.checked_add(3)?.checked_mul(l.rows)?.checked_add(18)
 }
 /// Task header, lineage and the largest available executor section.
 #[must_use]
-pub fn task_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
+fn task_nodes_in(l: &Limits, c: Shared, slot_nodes: u32) -> Option<u32> {
     let budget = crate::owner_authority::document_nodes(l.rows)?
         .checked_add(crate::owner_budget::budget_nodes())?
         .checked_add(l.rows.checked_mul(4)?)?
         .checked_add(4)?;
-    let activity = conversation_nodes(l, 0)?
-        .checked_add(runtime(l)?)?
-        .checked_add(card_nodes(l)?.checked_mul(2)?)?
+    let activity = conversation_nodes_in(l, c, 0)?
+        .checked_add(runtime_in(l, c)?)?
+        .checked_add(c.card.checked_mul(2)?)?
         // The Task owner keeps departing cards alongside each fresh bounded
         // source, so the shared working-set capacity bounds their union.
         .checked_add(super::proposal_nodes(l)?.checked_mul(l.objects)?)?
@@ -103,16 +113,16 @@ pub fn task_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
         .checked_add(2)?
         .checked_mul(5)?
         .checked_add(l.rows.checked_add(20)?.checked_mul(l.rows)?)?;
-    let section = plan(l)?.max(overview(l)?).max(history(l)?).max(budget).max(activity).max(panels);
-    card_nodes(l)?
-        .checked_add(chip_nodes(l)?.checked_mul(l.rows)?)?
+    let section = plan_in(l, c)?.max(overview_in(l, c)?).max(history_in(l)?).max(budget).max(activity).max(panels);
+    c.card
+        .checked_add(c.chip.checked_mul(l.rows)?)?
         .checked_add(section)?
         .checked_add(35)?
         .checked_add(slot_nodes)
 }
-fn goals(l: &Limits) -> Option<u32> {
-    let board = card_nodes(l)?
-        .checked_add(chip_nodes(l)?)?
+fn goals_in(l: &Limits, c: Shared) -> Option<u32> {
+    let board = c.card
+        .checked_add(c.chip)?
         .checked_add(crate::owner_budget::budget_nodes())?
         .checked_add(15)?
         .checked_mul(l.rows)?
@@ -126,28 +136,28 @@ fn goals(l: &Limits) -> Option<u32> {
         .checked_mul(l.projects.max(l.rows))?;
     board.checked_add(form)?.checked_add(34)
 }
-fn tasks(l: &Limits) -> Option<u32> {
-    card_nodes(l)?
-        .checked_add(chip_nodes(l)?)?
+fn tasks_in(l: &Limits, c: Shared) -> Option<u32> {
+    c.card
+        .checked_add(c.chip)?
         .checked_add(6)?
         .checked_mul(l.rows)?
         .checked_add(l.rows.checked_mul(2)?)?
         .checked_add(40)
 }
-fn notes(l: &Limits) -> Option<u32> {
-    card_nodes(l)?.checked_add(l.rows.checked_add(8)?)?.checked_mul(l.rows)?.checked_add(26)
+fn notes_in(l: &Limits, c: Shared) -> Option<u32> {
+    c.card.checked_add(l.rows.checked_add(8)?)?.checked_mul(l.rows)?.checked_add(26)
 }
-fn spend(l: &Limits) -> Option<u32> {
+fn spend_in(l: &Limits, c: Shared) -> Option<u32> {
     // Each past period has pools and task lines, each at the row bound.
-    card_nodes(l)?
-        .checked_add(chip_nodes(l)?)?
+    c.card
+        .checked_add(c.chip)?
         .checked_add(8)?
         .checked_mul(l.rows)?
         .checked_add(8)?
         .checked_mul(l.rows.checked_add(1)?)?
         .checked_add(10)
 }
-fn settings(l: &Limits) -> Option<u32> {
+fn settings_in(l: &Limits, c: Shared) -> Option<u32> {
     let policy = authority(l)?
         .checked_add(l.rows)?
         .checked_add(4)?
@@ -158,12 +168,12 @@ fn settings(l: &Limits) -> Option<u32> {
     // last batch's links and only the period boundaries the source supplied.
     let recurring = super::proposed_task_nodes(l)?
         .checked_mul(l.rows)?
-        .checked_add(chip_nodes(l)?)?
+        .checked_add(c.chip)?
         .checked_add(l.rows.checked_mul(2)?)?
         .checked_add(18)?
         .checked_mul(l.rows)?;
     policy
-        .checked_add(spend(l)?)?
+        .checked_add(spend_in(l, c)?)?
         .checked_add(crate::historical_pages::spend_nodes(l)?)?
         .checked_add(recurring)?
         .checked_add(l.rows.checked_mul(24)?)?
@@ -171,21 +181,21 @@ fn settings(l: &Limits) -> Option<u32> {
 }
 /// Project board, filtered windows and every settings subregion together.
 #[must_use]
-pub fn project_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
-    let overview = goals(l)?.checked_add(inbox_nodes(l, 0)?)?;
+fn project_nodes_in(l: &Limits, c: Shared, slot_nodes: u32) -> Option<u32> {
+    let overview = goals_in(l, c)?.checked_add(inbox_nodes_in(l, c, 0)?)?;
     overview
-        .max(goals(l)?)
-        .max(tasks(l)?)
-        .max(notes(l)?)
-        .max(settings(l)?)
+        .max(goals_in(l, c)?)
+        .max(tasks_in(l, c)?)
+        .max(notes_in(l, c)?)
+        .max(settings_in(l, c)?)
         .max(crate::owner_project::max_nodes(l)?)
         .checked_add(24)?
         .checked_add(slot_nodes)
 }
 /// Hosts, accounts, project capacity, refusals and deployment parts.
 #[must_use]
-pub fn deployment_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
-    let synthetic = chip_nodes(l)?
+fn deployment_nodes_in(l: &Limits, c: Shared, slot_nodes: u32) -> Option<u32> {
+    let synthetic = c.chip
         .checked_mul(l.rows)?
         .checked_mul(l.rows)?
         .checked_mul(3)?
@@ -201,11 +211,37 @@ pub fn not_the_work_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
 /// The largest jig page with every window and text at its bound.
 #[must_use]
 pub fn page_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
-    inbox_nodes(l, 0)?
-        .max(conversations_nodes(l, 0)?)
-        .max(task_nodes(l, 0)?)
-        .max(project_nodes(l, 0)?)
-        .max(deployment_nodes(l, 0)?)
+    let c = Shared::of(l)?;
+    inbox_nodes_in(l, c, 0)?
+        .max(conversations_nodes_in(l, c, 0)?)
+        .max(task_nodes_in(l, c, 0)?)
+        .max(project_nodes_in(l, c, 0)?)
+        .max(deployment_nodes_in(l, c, 0)?)
         .max(not_the_work_nodes(l, 0)?)
         .checked_add(slot_nodes)
 }
+
+#[must_use]
+pub fn inbox_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
+    inbox_nodes_in(l, Shared::of(l)?, slot_nodes)
+}
+#[must_use]
+pub fn conversations_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
+    conversations_nodes_in(l, Shared::of(l)?, slot_nodes)
+}
+#[must_use]
+pub fn conversation_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
+    conversation_nodes_in(l, Shared::of(l)?, slot_nodes)
+}
+#[must_use]
+pub fn task_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
+    task_nodes_in(l, Shared::of(l)?, slot_nodes)
+}
+#[must_use]
+pub fn project_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
+    project_nodes_in(l, Shared::of(l)?, slot_nodes)
+}
+#[must_use]
+pub fn deployment_nodes(l: &Limits, slot_nodes: u32) -> Option<u32> {
+    deployment_nodes_in(l, Shared::of(l)?, slot_nodes)
+}
```
