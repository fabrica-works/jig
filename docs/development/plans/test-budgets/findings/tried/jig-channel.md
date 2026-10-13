# jig: the channel audit's tried changes

Evidence for the plan's sessions, against jig `d91a89c45`: what the audit measured, not code to apply as it is. An increment's Shape is the session file's.

```diff
diff --git a/crates/jig-protocol-config/src/overrides.rs b/crates/jig-protocol-config/src/overrides.rs
index 0f3052bab..69bd27971 100644
--- a/crates/jig-protocol-config/src/overrides.rs
+++ b/crates/jig-protocol-config/src/overrides.rs
@@ -549,12 +549,13 @@ fn members(
         return Ok(());
     }
     next = at.checked_add(1).ok_or(What::Bound)?;
+    let mut fits = Fits { hub: None, core: None };
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
@@ -562,17 +563,23 @@ fn members(
     Ok(())
 }
 
-fn valid(limits: &Limits, name: &[u8]) -> bool {
+/// Each family's worst case over the overridden limits, computed once when first asked.
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
diff --git a/crates/jig-protocol-store/src/transcripts/frame.rs b/crates/jig-protocol-store/src/transcripts/frame.rs
index b8500cae5..5d32ee098 100644
--- a/crates/jig-protocol-store/src/transcripts/frame.rs
+++ b/crates/jig-protocol-store/src/transcripts/frame.rs
@@ -43,14 +43,27 @@ pub fn decode(bytes: &[u8], limits: &codec::Limits) -> Result<core::TurnRecord,
         Err(_) => Err(Unread::Record),
     }
 }
+/// CRC-32C's reflected table: one lookup per byte, not eight shifts.
+const TABLE: [u32; 256] = {
+    let mut table = [0_u32; 256];
+    let mut index = 0_u32;
+    while index < 256 {
+        let mut crc = index;
+        let mut bit = 0;
+        while bit < 8 {
+            crc = if crc & 1 == 0 { crc >> 1_u32 } else { (crc >> 1_u32) ^ 0x82f6_3b78 };
+            bit += 1;
+        }
+        table[index as usize] = crc;
+        index += 1;
+    }
+    table
+};
 #[must_use]
 pub fn crc32c(bytes: &[u8]) -> u32 {
     let mut crc = !0_u32;
     for byte in bytes {
-        crc ^= u32::from(*byte);
-        for _ in 0_u8..8 {
-            crc = if crc & 1 == 0 { crc >> 1_u32 } else { (crc >> 1_u32) ^ 0x82f6_3b78 };
-        }
+        crc = (crc >> 8_u32) ^ TABLE[usize::from((crc as u8) ^ *byte)];
     }
     !crc
 }
diff --git a/testing/jig-test-service/src/engine.rs b/testing/jig-test-service/src/engine.rs
index 49c0c2d0c..ea98efb6b 100644
--- a/testing/jig-test-service/src/engine.rs
+++ b/testing/jig-test-service/src/engine.rs
@@ -31,6 +31,8 @@ pub struct Engine {
     pending: Option<Box<root::Event>>,
     ready: bool,
     pub(crate) first_periods: skein_lib::List<(u32, u64)>,
+    /// Inputs handed to the root and outputs taken from it, for `Service::root_steps`.
+    pub(crate) root_steps: u64,
 }
 
 pub(crate) struct Children {
@@ -84,6 +86,7 @@ impl Engine {
             pending: None,
             ready: true,
             first_periods: skein_lib::List::with_capacity(limits.core.authority.projects),
+            root_steps: 0,
         }
     }
     pub const fn parties_in(&mut self) -> &mut Queue<core::Event> {
@@ -118,6 +121,7 @@ impl Engine {
             }
     }
     pub(crate) fn step(&mut self, event: root::Event) {
+        self.root_steps = self.root_steps.wrapping_add(1);
         let watcher = match &mut self.clients {
             Some(clients) => clients.root_event(&event),
             None => None,
@@ -350,6 +354,7 @@ pub fn iterate(engine: &mut Engine, now: Time, wall: Wall) {
         let event = engine.events.pop().expect("ready component input");
         engine.step(event);
     } else if let Some(event) = engine.parties.pop() {
+        engine.root_steps = engine.root_steps.wrapping_add(1);
         if let Err(event) = root::try_step(&mut engine.domain, &engine.env, root::Event::Core(event)) {
             engine.outputs.push(PartyOutput::Busy(Box::new(event)));
         }
@@ -378,6 +383,9 @@ fn release(engine: &mut Engine) {
             break;
         }
         root::release(&mut engine.domain, &engine.env, &mut engine.requests);
+        if !engine.requests.is_empty() {
+            engine.root_steps = engine.root_steps.wrapping_add(1);
+        }
         if !engine.requests.is_empty() || engine.domain.quiescent() {
             engine.blocked = false;
         }
diff --git a/testing/jig-test-service/src/lib.rs b/testing/jig-test-service/src/lib.rs
index 010bcbd14..ba22694d5 100644
--- a/testing/jig-test-service/src/lib.rs
+++ b/testing/jig-test-service/src/lib.rs
@@ -127,6 +127,16 @@ impl Service {
         }
         roots.finish()
     }
+    /// Inputs handed to the roots and outputs taken from them so far: a root's
+    /// state changes only with one, so a check of the roots' state runs when it moves.
+    #[must_use]
+    pub fn root_steps(&self) -> u64 {
+        let mut steps = self.engine.as_ref().map_or(0, |engine| engine.root_steps);
+        for worker in &self.workers {
+            steps = steps.wrapping_add(worker.root_steps);
+        }
+        steps
+    }
     #[must_use]
     pub const fn end(&self) -> Option<&End> {
         match &self.engine {
diff --git a/testing/jig-test-service/src/route.rs b/testing/jig-test-service/src/route.rs
index 9d8d30ac5..a0ccccecd 100644
--- a/testing/jig-test-service/src/route.rs
+++ b/testing/jig-test-service/src/route.rs
@@ -121,6 +121,7 @@ fn restart(engine: &mut Engine, step: core::RestartStep) {
         }
         core::RestartStep::LoadCore | core::RestartStep::LoadInbox | core::RestartStep::RestoreConnector { .. } => {
             let census = if step == core::RestartStep::LoadInbox {
+                engine.root_steps = engine.root_steps.wrapping_add(1);
                 Some(root::inbox_census(&mut engine.domain, &engine.env.limits))
             } else {
                 None
diff --git a/testing/jig-test-service/src/store.rs b/testing/jig-test-service/src/store.rs
index 40f435691..b4058735e 100644
--- a/testing/jig-test-service/src/store.rs
+++ b/testing/jig-test-service/src/store.rs
@@ -467,6 +467,7 @@ pub(super) fn answer(engine: &mut crate::Engine) {
                 if row.key() == root::Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment)) {
                     engine.first_periods.truncate(0);
                 }
+                engine.root_steps = engine.root_steps.wrapping_add(1);
                 if !root::restore_record(&mut engine.domain, &engine.env, row) {
                     crate::stop::invalid_restore(engine);
                 }
diff --git a/testing/jig-test-service/src/worker.rs b/testing/jig-test-service/src/worker.rs
index 63377c832..bd9dba0e4 100644
--- a/testing/jig-test-service/src/worker.rs
+++ b/testing/jig-test-service/src/worker.rs
@@ -20,6 +20,7 @@ pub struct WorkerRuntime {
 #[derive(Debug)]
 pub struct Worker {
     pub(super) root: root::Domain,
+    pub(super) root_steps: u64,
     env: Env<root::Limits>,
     agent: Agent,
     link: link::Component,
@@ -102,6 +103,7 @@ impl Worker {
             .ok_or(problem(index, b"slots", What::Bound))?;
         Ok(Self {
             root: root::Domain::new(&root_limits),
+            root_steps: 0,
             env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: root_limits },
             agent: Agent {
                 endpoints: agent.endpoints.clone(),
@@ -321,6 +323,7 @@ impl Worker {
         self.application.expire(now);
     }
     pub(super) fn step(&mut self, event: root::Event) {
+        self.root_steps = self.root_steps.wrapping_add(1);
         root::step(&mut self.root, &self.env, event, &mut self.out);
         for _ in 0..self.out.capacity() {
             let Some(request) = self.out.pop() else { break };
@@ -334,14 +337,17 @@ impl Worker {
                 root::Request::Process(request) => self.process(*request),
                 root::Request::Decode { agent, run, start } => {
                     let start = worker_route::decode(run, *start, &self.agent);
+                    self.root_steps = self.root_steps.wrapping_add(1);
                     root::step(&mut self.root, &self.env, root::Event::Decoded { agent, start }, &mut self.out);
                 }
                 root::Request::Encode { owner, value } => {
                     let encoded = worker_route::encode(value, &self.agent.limits, &self.agent.endpoints);
+                    self.root_steps = self.root_steps.wrapping_add(1);
                     root::step(&mut self.root, &self.env, root::Event::Encoded { owner, encoded }, &mut self.out);
                 }
                 root::Request::Fact { owner, body } => {
                     let activity = worker_route::activity(&body, &self.agent.limits);
+                    self.root_steps = self.root_steps.wrapping_add(1);
                     root::step(&mut self.root, &self.env, root::Event::Activity { owner, activity }, &mut self.out);
                 }
             }
diff --git a/tests/channel/src/fixtures.rs b/tests/channel/src/fixtures.rs
index 59fd50c02..71f280c34 100644
--- a/tests/channel/src/fixtures.rs
+++ b/tests/channel/src/fixtures.rs
@@ -422,15 +422,18 @@ pub(crate) fn engine_message_document() -> Vec<u8> {
     engine_document()
 }
 
-/// Match the fixture's final admitted core bounds, once per engine instance.
+/// Match the fixture's final admitted core bounds, parsed once per test process.
 #[must_use]
 pub fn store_limits() -> jig_test_service::limits::Limits {
-    let bounds = jig_protocol_config::DocumentLimits {
-        bytes: u32::try_from(ENGINE.len()).expect("fixture document size"),
-        tokens: u32::try_from(ENGINE.len()).expect("fixture document size"),
-        depth: 32,
-    };
-    let document = jig_protocol_config::document(ENGINE, &bounds).expect("fixture document");
-    let engine = jig_protocol_config::engine(&document, 0).expect("fixture engine");
-    jig_test_service::limits::for_profile(jig_protocol_config::Profile::Small, &engine.limits)
+    static LIMITS: std::sync::OnceLock<jig_test_service::limits::Limits> = std::sync::OnceLock::new();
+    *LIMITS.get_or_init(|| {
+        let bounds = jig_protocol_config::DocumentLimits {
+            bytes: u32::try_from(ENGINE.len()).expect("fixture document size"),
+            tokens: u32::try_from(ENGINE.len()).expect("fixture document size"),
+            depth: 32,
+        };
+        let document = jig_protocol_config::document(ENGINE, &bounds).expect("fixture document");
+        let engine = jig_protocol_config::engine(&document, 0).expect("fixture engine");
+        jig_test_service::limits::for_profile(jig_protocol_config::Profile::Small, &engine.limits)
+    })
 }
diff --git a/tests/channel/src/referee.rs b/tests/channel/src/referee.rs
index b39dd9e5c..e15105596 100644
--- a/tests/channel/src/referee.rs
+++ b/tests/channel/src/referee.rs
@@ -530,13 +530,13 @@ impl Referee {
         for proc in procs {
             if let Proc::Engine(host) | Proc::Worker(host) = proc
                 && !grants.is_empty()
-                && self.roots_observed.get(&host.revision.0) != Some(&host.revision.1)
+                && self.roots_observed.get(&host.revision.0) != Some(&host.service.root_steps())
             {
                 use std::fmt::Write;
                 self.formatted.clear();
                 write!(&mut self.formatted, "{:?}", Roots(&host.service)).expect("root state");
                 self.rules.roots(&self.formatted, &grants);
-                self.roots_observed.insert(host.revision.0, host.revision.1);
+                self.roots_observed.insert(host.revision.0, host.service.root_steps());
             }
         }
         assert_eq!(self.rules.verdict(), skein_world::domain::Verdict::Passed);
diff --git a/tests/channel/src/relay.rs b/tests/channel/src/relay.rs
index 401c90bff..b62bc7c0e 100644
--- a/tests/channel/src/relay.rs
+++ b/tests/channel/src/relay.rs
@@ -66,6 +66,7 @@ pub(crate) struct Peer {
     refusal: Option<jig_protocol_worker_wire::MessageRefusal>,
     recorded: Option<crate::world::Recorded>,
     frame_reads: bool,
+    limits: skein_channel::Limits,
 }
 impl Peer {
     pub(crate) fn opening(case: crate::stories::Opening) -> Self {
@@ -155,6 +156,7 @@ impl Peer {
             refusal: None,
             recorded: None,
             frame_reads: false,
+            limits,
         }
     }
     pub(crate) fn shutdown(&mut self) {
@@ -212,9 +214,8 @@ impl Peer {
                 return;
             }
             if header.kind == 3 {
-                let (_, limits) = crate::fixtures::peer_bounds();
                 let control =
-                    skein_channel::decode_control(header.kind, &self.input[8..end], &limits).expect("refusal");
+                    skein_channel::decode_control(header.kind, &self.input[8..end], &self.limits).expect("refusal");
                 let skein_channel::Control::Refuse { reason, .. } = control else { panic!("refusal kind") };
                 self.refused = Some(reason);
             }
@@ -320,9 +321,8 @@ impl skein_shell::Host for Peer {
             });
         }
         if self.heartbeat.is_some_and(|at| now >= at) && !self.closing {
-            let (_, limits) = crate::fixtures::peer_bounds();
             self.scripted
-                .play(skein_channel::control_frame(&skein_channel::Control::Ping, &limits).expect("peer heartbeat"));
+                .play(skein_channel::control_frame(&skein_channel::Control::Ping, &self.limits).expect("peer heartbeat"));
             self.heartbeats += 1;
             self.heartbeat = Some(now.saturating_add(skein_lib::Duration::from_secs(1)));
             self.demand();
diff --git a/tests/channel/src/world/store.rs b/tests/channel/src/world/store.rs
index 4f13f4770..1fdf06d44 100644
--- a/tests/channel/src/world/store.rs
+++ b/tests/channel/src/world/store.rs
@@ -89,7 +89,7 @@ impl Capture {
         if let Op::Write { fd, bytes, from: 0, .. } = op
             && self.path(*fd).is_some_and(|path| path.starts_with(b"store/"))
         {
-            let frames = frames(bytes);
+            let frames = frames(bytes, 0);
             let turn = frames.iter().any(|(_, ops)| {
                 ops.iter().any(|op| match op {
                     skein_kv::Op::Put { key, .. } => {
@@ -148,7 +148,7 @@ impl Store {
                     file[at..at + bytes.len()].copy_from_slice(&bytes);
                 }
                 Observed::Pending(bytes) => {
-                    self.pending = frames(&bytes)
+                    self.pending = frames(&bytes, self.applied)
                         .into_iter()
                         .filter(|(number, _)| *number > self.applied)
                         .map(|(number, ops)| self.commit(number, ops, limits, connector))
@@ -156,7 +156,7 @@ impl Store {
                 }
                 Observed::Recovered(path) => {
                     if let Some(bytes) = self.files.get(&path) {
-                        self.recovered.extend(frames(bytes).into_iter().map(|(number, _)| number));
+                        self.recovered.extend(frames(bytes, 0).into_iter().map(|(number, _)| number));
                     }
                     committed.extend(self.durable(&path, limits, connector));
                 }
@@ -171,7 +171,7 @@ impl Store {
     fn durable(&mut self, path: &[u8], limits: &codec::Limits, connector: &jig_test_connector::Limits) -> Vec<Commit> {
         let mut committed = Vec::new();
         let Some(bytes) = self.files.get(path) else { return committed };
-        let frames = frames(bytes);
+        let frames = frames(bytes, self.applied);
         for (number, ops) in frames {
             if number <= self.applied {
                 continue;
@@ -248,7 +248,8 @@ impl Store {
         Commit { number, writes }
     }
 }
-fn frames(bytes: &[u8]) -> Vec<(u64, Box<[skein_kv::Op]>)> {
+/// The whole frames past `after`: a frame at or below it was checked when it was applied, and is skipped by its length.
+fn frames(bytes: &[u8], after: u64) -> Vec<(u64, Box<[skein_kv::Op]>)> {
     let mut reader = Reader::new(bytes);
     let mut frames = Vec::new();
     while reader.remaining() >= 26 {
@@ -261,6 +262,10 @@ fn frames(bytes: &[u8]) -> Vec<(u64, Box<[skein_kv::Op]>)> {
         if reader.remaining() < body + 4 {
             break;
         }
+        if number <= after {
+            reader.skip(body + 4).expect("an applied frame's body and checksum");
+            continue;
+        }
         let end = (bytes.len() - reader.remaining() as usize) + body as usize;
         let mut ops = Vec::new();
         for _ in 0..count {
```
