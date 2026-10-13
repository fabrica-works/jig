# jig: the overrides' worst case once, tried

Evidence for the plan's sessions, against jig `d91a89c45`: what the audit measured, not code to apply as it is. An increment's Shape is the session file's.

```diff
diff --git a/crates/jig-protocol-config/src/overrides.rs b/crates/jig-protocol-config/src/overrides.rs
index 0f3052bab..4c0ad23a1 100644
--- a/crates/jig-protocol-config/src/overrides.rs
+++ b/crates/jig-protocol-config/src/overrides.rs
@@ -549,12 +549,13 @@ fn members(
         return Ok(());
     }
     next = at.checked_add(1).ok_or(What::Bound)?;
+    let mut fits = Fits::default();
     for _ in at..end {
         if next == end {
             break;
         }
         let name = document.text(document.token(next).ok_or(What::Kind)?).ok_or(What::Kind)?;
-        if !valid(limits, name) {
+        if !valid(limits, name, &mut fits) {
             problems.push(path.member(name), What::Override);
         }
         next = read::after(document, next.checked_add(1).ok_or(What::Bound)?)?;
@@ -562,17 +563,24 @@ fn members(
     Ok(())
 }
 
-fn valid(limits: &Limits, name: &[u8]) -> bool {
+/// Whether the final limits' worst cases fit, each computed once (audit experiment).
+#[derive(Default)]
+struct Fits {
+    hub: Option<bool>,
+    core: Option<bool>,
+}
+
+fn valid(limits: &Limits, name: &[u8], fits: &mut Fits) -> bool {
     let Some(dot) = bytes::find(name, b".") else {
         return false;
     };
     match name.get(..dot) {
-        Some(b"hub") => jig_domain_host::worst_case(&limits.hub).is_some(),
+        Some(b"hub") => *fits.hub.get_or_insert_with(|| jig_domain_host::worst_case(&limits.hub).is_some()),
         Some(b"link") => true,
         Some(
             b"core" | b"tasks" | b"people" | b"fleet" | b"authority" | b"brief" | b"accounts" | b"notes" | b"views"
             | b"documents",
-        ) => jig_domain_core::worst_case(&limits.core).is_some(),
+        ) => *fits.core.get_or_insert_with(|| jig_domain_core::worst_case(&limits.core).is_some()),
         Some(_) | None => false,
     }
 }
```
