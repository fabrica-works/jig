# smith: skein-base64, tried

Evidence for the plan's sessions, against smith `0305f2fd`: what the audit measured, not code to apply as it is. An increment's Shape is the session file's.

```diff
--- /srv/data/git/runner/.cargo/git/checkouts/skein-336d5ae5eecfa0f9/cb3cefb/crates/skein-lib/src/base64.rs	2026-10-13 21:55:57.789434423 +0100
+++ crates/skein-lib/src/base64.rs	2026-10-13 22:19:32.589447800 +0100
@@ -79,6 +79,19 @@
     Ok(())
 }
 
+/// Each byte's six-bit value in an alphabet, or 64 for a byte outside it.
+const fn values(symbols: &[u8; 64]) -> [u8; 256] {
+    let mut table = [64_u8; 256];
+    let mut index = 0;
+    while index < 64 {
+        table[symbols[index] as usize] = index as u8;
+        index += 1;
+    }
+    table
+}
+const STANDARD_VALUES: [u8; 256] = values(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/");
+const URL_VALUES: [u8; 256] = values(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_");
+
 #[derive(Clone, Copy, Debug)]
 struct State {
     alphabet: Alphabet,
@@ -117,18 +130,14 @@
         if self.padding != 0 {
             return Err(Error::Padding { at });
         }
-        let table: &[u8] = match self.alphabet {
-            Alphabet::Standard => b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
-            Alphabet::Url => b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_",
+        let table: &[u8; 256] = match self.alphabet {
+            Alphabet::Standard => &STANDARD_VALUES,
+            Alphabet::Url => &URL_VALUES,
+        };
+        let value = match table.get(usize::from(byte)).copied() {
+            Some(value) if value < 64 => usize::from(value),
+            Some(_) | None => return Err(Error::Symbol { at }),
         };
-        let mut value = None;
-        for (index, symbol) in table.iter().enumerate() {
-            if *symbol == byte {
-                value = Some(index);
-                break;
-            }
-        }
-        let value = value.ok_or(Error::Symbol { at })?;
         self.carry = (self.carry << 6_u32) | u16::try_from(value).expect("six bit digit");
         self.bits = self.bits.checked_add(6).expect("at most twelve bits");
         if self.bits < 8 {
```
