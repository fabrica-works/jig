# smith: parallel-and-uid, tried

Evidence for the plan's sessions, against smith `0305f2fd`: what the audit measured, not code to apply as it is. An increment's Shape is the session file's.

```diff
diff --git a/.config/nextest.toml b/.config/nextest.toml
index 996b939d..784571b5 100644
--- a/.config/nextest.toml
+++ b/.config/nextest.toml
@@ -31,9 +31,6 @@ slow-timeout = { period = "60s" }
 [test-groups.real-local]
 max-threads = 1
 
-[[profile.default.overrides]]
-filter = "package(smith-real-world) or binary(end_to_end)"
-test-group = "real-local"
 
 # Real backends are opt-in and never part of the merge gate.
 [profile.live]
diff --git a/tests/agent-process/src/fake.rs b/tests/agent-process/src/fake.rs
index b5a5f31a..681ff9f6 100644
--- a/tests/agent-process/src/fake.rs
+++ b/tests/agent-process/src/fake.rs
@@ -6,6 +6,38 @@ use skein_fake_llm_protocol::{documents, provider};
 use skein_fake_peers::{Limits, Transport, llm};
 use skein_lib::Duration;
 
+static REAL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
+
+/// Real tiers call this once: from then on this process's peers bind its own
+/// loopback address, so stories in different processes never share a port.
+pub fn real_loopback() {
+    REAL.store(true, std::sync::atomic::Ordering::Relaxed);
+}
+
+/// 127.1.0.0 plus the process ID (below 2^22) in a real tier: distinct for
+/// every process running at once. 127.0.0.1 in the simulated tiers.
+#[must_use]
+pub fn loopback() -> std::net::Ipv4Addr {
+    if REAL.load(std::sync::atomic::Ordering::Relaxed) {
+        std::net::Ipv4Addr::from(0x7f01_0000_u32 + std::process::id())
+    } else {
+        std::net::Ipv4Addr::LOCALHOST
+    }
+}
+
+/// The sign-in redirect port: the product binds 127.0.0.1 only, so a real
+/// tier takes a port the kernel had free; 2345 in the simulated tiers.
+#[must_use]
+pub fn redirect_port() -> u16 {
+    static PORT: std::sync::OnceLock<u16> = std::sync::OnceLock::new();
+    if !REAL.load(std::sync::atomic::Ordering::Relaxed) {
+        return 2345;
+    }
+    *PORT.get_or_init(|| {
+        std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap().local_addr().unwrap().port()
+    })
+}
+
 /// Bounded loopback transport and observation settings for Smith's peers.
 #[must_use]
 pub fn limits() -> Limits {
@@ -118,7 +150,7 @@ fn configured_bounds(
     protocol.documents.openai.output_items = 256;
     protocol.documents.anthropic.output_items = 256;
     llm::Peer::new(
-        (std::net::Ipv4Addr::LOCALHOST, if transport == Transport::Tls { 34_443 } else { 443 }).into(),
+        (loopback(), if transport == Transport::Tls { 34_443 } else { 443 }).into(),
         transport,
         limits(),
         provider::Config {
diff --git a/tests/end-to-end/end_to_end.rs b/tests/end-to-end/end_to_end.rs
index bf007afe..b7fa9ed2 100644
--- a/tests/end-to-end/end_to_end.rs
+++ b/tests/end-to-end/end_to_end.rs
@@ -203,9 +203,11 @@ fn smith_on_a_terminal_signs_in_and_commits_in_place() {
     let scratch = Scratch::new();
     let mut settings: toml::Value =
         toml::from_str(std::str::from_utf8(&support::fake_provider_settings(&scratch)).unwrap()).unwrap();
+    let host = smith_agent_process_world::fake::loopback();
+    let redirect = smith_agent_process_world::fake::redirect_port();
     settings["accounts"]["codex"] = toml::Value::try_from(serde_json::json!({
-        "source":"sign_in", "client_id":"client", "authorization_url":"http://127.0.0.1:34444/authorize",
-        "token_url":"http://127.0.0.1:34444/token", "redirect_uri":"http://127.0.0.1:2345/callback",
+        "source":"sign_in", "client_id":"client", "authorization_url":format!("http://{host}:34444/authorize"),
+        "token_url":format!("http://{host}:34444/token"), "redirect_uri":format!("http://127.0.0.1:{redirect}/callback"),
         "scope":"read", "token_format":"form"
     }))
     .unwrap();
@@ -230,7 +232,7 @@ fn smith_on_a_terminal_signs_in_and_commits_in_place() {
         ],
     );
     assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
-    assert!(String::from_utf8_lossy(&ran.stdout).contains("Sign in: http://127.0.0.1:34444/authorize"));
+    assert!(String::from_utf8_lossy(&ran.stdout).contains(&format!("Sign in: http://{host}:34444/authorize")));
     assert!(scratch.path().join("state/smith/tokens/636f646578.token").is_file());
     let checkout = scratch.checkout();
     let after = checkout.head().unwrap();
@@ -903,7 +905,8 @@ fn smith_login_signs_in_against_the_fake_issuer_and_exits_0() {
     let scratch = Scratch::new();
     login_settings(
         &scratch,
-        "client_id='client'\nauthorization_url='http://127.0.0.1:34444/authorize'\ntoken_url='http://127.0.0.1:34444/token'\nredirect_uri='http://127.0.0.1:2345/callback'\nscope='read'\ntoken_format='form'\n",
+        &format!("client_id='client'\nauthorization_url='http://{host}:34444/authorize'\ntoken_url='http://{host}:34444/token'\nredirect_uri='http://127.0.0.1:{redirect}/callback'\nscope='read'\ntoken_format='form'\n",
+            host = smith_agent_process_world::fake::loopback(), redirect = smith_agent_process_world::fake::redirect_port()),
     );
     let ran = support::login::sign_in(&scratch);
     assert_eq!(ran.code, 0);
diff --git a/tests/end-to-end/support/exec.rs b/tests/end-to-end/support/exec.rs
index 636893c9..21c4d022 100644
--- a/tests/end-to-end/support/exec.rs
+++ b/tests/end-to-end/support/exec.rs
@@ -28,7 +28,7 @@ pub fn fake_provider_settings(scratch: &Scratch) -> Vec<u8> {
     let document = serde_json::json!({"tokens": {"access_token": std::str::from_utf8(&token).expect("scratch token text"), "account_id": "acc", "refresh_token": "UNISSUED_NEVER_COPY"}});
     std::fs::write(scratch.path().join("auth.json"), serde_json::to_vec(&document).expect("scratch login encoding"))
         .expect("scratch login file");
-    format!("model='codex/gpt-6-luna'\neffort='low'\ninstructions='@local-shell Assist'\n[accounts.codex]\nsource='borrow'\nfile='{}'\nformat='codex'\nprogram='codex'\n[endpoints.codex]\naddress='127.0.0.1:34443'\nserver_name='skein.test'\ntrust.system=false\ntrust.der=['{}']\n", scratch.path().join("auth.json").display(), scratch.path().join("root.der").display()).into_bytes()
+    format!("model='codex/gpt-6-luna'\neffort='low'\ninstructions='@local-shell Assist'\n[accounts.codex]\nsource='borrow'\nfile='{}'\nformat='codex'\nprogram='codex'\n[endpoints.codex]\naddress='{}:34443'\nserver_name='skein.test'\ntrust.system=false\ntrust.der=['{}']\n", scratch.path().join("auth.json").display(), smith_agent_process_world::fake::loopback(), scratch.path().join("root.der").display()).into_bytes()
 }
 
 #[must_use]
diff --git a/tests/end-to-end/support/mod.rs b/tests/end-to-end/support/mod.rs
index 5fc8c592..b37547c2 100644
--- a/tests/end-to-end/support/mod.rs
+++ b/tests/end-to-end/support/mod.rs
@@ -266,15 +266,17 @@ pub fn settings(scratch: &Scratch, scenario: &Scenario) -> Box<[u8]> {
         [deployment]\nmemory=4294967296\nconversations=2\nllm_pool=134217728\ngroup_stop=1\n\
         [models.'fake/fake']\nwindow=2097152\noutput=65536\nreasoning_item=2048\nhead=60\nidle=30\n\
         input_price=1.25\ncached_price=0.125\noutput_price=10.0\nprice_unit=1000000\n\
-        [endpoints.fake]\npreset='codex'\naccount='fake'\naddress='127.0.0.1:34443'\nserver_name='skein.test'\n\
+        [endpoints.fake]\npreset='codex'\naccount='fake'\naddress='{}:34443'\nserver_name='skein.test'\n\
         identity='plain'\ntrust.system=false\ntrust.der=['{}']\n\
         [environment]\ninherit=[]\n[trace]\npath='{}'\ncapture='calls'\n",
+        smith_agent_process_world::fake::loopback(),
         scratch.path().join("root.der").display(),
         scratch.path().join("agent-trace.jsonl").display()
     );
     if scenario.launch.authenticated {
-        text.push_str("[accounts.fake]\nsource='sign_in'\nclient_id='client'\nauthorization_url='http://127.0.0.1:34444/authorize'\n\
-            token_url='http://127.0.0.1:34444/token'\nredirect_uri='http://127.0.0.1:2345/callback'\nscope='read'\ntoken_format='form'\n");
+        write!(text, "[accounts.fake]\nsource='sign_in'\nclient_id='client'\nauthorization_url='http://{host}:34444/authorize'\n\
+            token_url='http://{host}:34444/token'\nredirect_uri='http://127.0.0.1:{redirect}/callback'\nscope='read'\ntoken_format='form'\n",
+            host = smith_agent_process_world::fake::loopback(), redirect = smith_agent_process_world::fake::redirect_port()).expect("fixture string");
     } else {
         write!(
             text,
diff --git a/tests/local-process/src/process.rs b/tests/local-process/src/process.rs
index e848334b..d793d8a7 100644
--- a/tests/local-process/src/process.rs
+++ b/tests/local-process/src/process.rs
@@ -123,16 +123,18 @@ fn issuer_with_token(transport: skein_fake_peers::Transport, port: u16, access_t
         rotations: 4,
         plans: 4,
     };
+    let host = smith_agent_process_world::fake::loopback();
+    let redirect = smith_agent_process_world::fake::redirect_port();
     let mut issuer = Issuer::new(
-        (std::net::Ipv4Addr::LOCALHOST, port).into(),
+        (host, port).into(),
         transport,
         smith_agent_process_world::fake::limits(),
         fake_oauth::Config {
-            authorization_url: format!("{scheme}://127.0.0.1:{port}/authorize").into_bytes().into(),
-            token_endpoint: format!("{scheme}://127.0.0.1:{port}/token").into_bytes().into(),
+            authorization_url: format!("{scheme}://{host}:{port}/authorize").into_bytes().into(),
+            token_endpoint: format!("{scheme}://{host}:{port}/token").into_bytes().into(),
             client_id: b"client".as_slice().into(),
             client_secret: None,
-            redirect_uri: b"http://127.0.0.1:2345/callback".as_slice().into(),
+            redirect_uri: format!("http://127.0.0.1:{redirect}/callback").into_bytes().into(),
             refresh_token: b"refresh-old".as_slice().into(),
         },
         limits,
diff --git a/tests/local-process/src/process/configuration.rs b/tests/local-process/src/process/configuration.rs
index b55c4165..2887ba6a 100644
--- a/tests/local-process/src/process/configuration.rs
+++ b/tests/local-process/src/process/configuration.rs
@@ -289,7 +289,7 @@ pub fn agent_document(launch: &Launch) -> serde_json::Value {
     }
     let mut endpoint = serde_json::json!({"name":if launch.choices {"main"} else {"fake"},"number":0,"dialect":0,
         "account":0,"provider":"codex","connect":1000,"handshake":1000,"models":models,
-        "identity":"plain","address":if launch.tls {"127.0.0.1:34443"} else {"127.0.0.1:443"},
+        "identity":"plain","address":if launch.tls {format!("{}:34443", smith_agent_process_world::fake::loopback())} else {"127.0.0.1:443".into()},
         "transport":if launch.tls {"tls"} else {"plaintext"},
         "headers":[{"name":"originator","value":"skein-world"}]});
     if launch.tls {
@@ -326,15 +326,18 @@ pub fn prepared_accounts(
     let mut sources = skein_lib::List::with_capacity(1);
     let source = if authenticated {
         let port = if tls { 34_444 } else { 444 };
+        let host = smith_agent_process_world::fake::loopback();
         let registration = skein_oauth::Registration {
-            authorization_url: format!("{}://127.0.0.1:{port}/authorize", if tls { "https" } else { "http" })
+            authorization_url: format!("{}://{host}:{port}/authorize", if tls { "https" } else { "http" })
                 .into_bytes()
                 .into(),
-            token_endpoint: format!("{}://127.0.0.1:{port}/token", if tls { "https" } else { "http" })
+            token_endpoint: format!("{}://{host}:{port}/token", if tls { "https" } else { "http" })
                 .into_bytes()
                 .into(),
             client_id: b"client".as_slice().into(),
-            redirect_uri: b"http://127.0.0.1:2345/callback".as_slice().into(),
+            redirect_uri: format!("http://127.0.0.1:{}/callback", smith_agent_process_world::fake::redirect_port())
+                .into_bytes()
+                .into(),
             scope: b"read".as_slice().into(),
             wire: skein_oauth::WireFormat::Form,
             client_secret: None,
@@ -349,7 +352,7 @@ pub fn prepared_accounts(
             file: b"0.json".as_slice().into(),
             registration,
             endpoint: skein_oauth_accounts::Endpoint {
-                address: (std::net::Ipv4Addr::LOCALHOST, port).into(),
+                address: (host, port).into(),
                 transport: if tls {
                     skein_oauth_accounts::Transport::Tls {
                         server_name: skein_tls_world::pki::name(),
diff --git a/tests/local-process/src/world.rs b/tests/local-process/src/world.rs
index 9739170e..4d17879a 100644
--- a/tests/local-process/src/world.rs
+++ b/tests/local-process/src/world.rs
@@ -600,7 +600,7 @@ impl World {
         behavior: (bool, bool, bool),
     ) -> Self {
         let (interrupt_on_query, keep_facts, change) = behavior;
-        let mut disk = Filesystem::new();
+        let mut disk = Filesystem::for_user(skein_shell::effective_user());
         std::fs::create_dir_all(files.path.join("chat")).expect("chat fixture");
         let mut items =
             vec![skein_fake_machine::Item::directory(b".smith-test"), skein_fake_machine::Item::directory(b"chat")];
diff --git a/tests/real/Cargo.toml b/tests/real/Cargo.toml
index 24e0c219..c0801d3c 100644
--- a/tests/real/Cargo.toml
+++ b/tests/real/Cargo.toml
@@ -19,6 +19,7 @@ smith-local-shell.workspace = true
 smith-agent-shell.workspace = true
 smith-local-domain.workspace = true
 smith-local-process-world.workspace = true
+smith-agent-process-world.workspace = true
 serde_json = "1"
 
 [lints.rust]
diff --git a/tests/real/src/lib.rs b/tests/real/src/lib.rs
index acd33441..e522fb4a 100644
--- a/tests/real/src/lib.rs
+++ b/tests/real/src/lib.rs
@@ -77,6 +77,7 @@ impl Scratch {
         for (tool, path) in [("git", "/usr/bin/git"), ("rg", "/usr/bin/rg"), ("sh", "/bin/sh")] {
             assert!(Path::new(path).is_file(), "real world requires {tool} at {path}");
         }
+        smith_agent_process_world::fake::real_loopback();
         let files = Files::new();
         let repo = files.path().join("repo");
         std::fs::create_dir(&repo).expect("scratch repository");
```
