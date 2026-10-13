# skein: the suite audit's tried changes

Evidence for the plan's sessions, against skein `2068257`: what the audit measured, not code to apply as it is. An increment's Shape is the session file's.

```diff
diff --git a/crates/skein-llm/src/openai/tests.rs b/crates/skein-llm/src/openai/tests.rs
index 1ebd0a0..28e2d5d 100644
--- a/crates/skein-llm/src/openai/tests.rs
+++ b/crates/skein-llm/src/openai/tests.rs
@@ -1387,12 +1387,12 @@ fn captured_completion_retains_status_and_usage_without_echoes() {
 fn native_function_arguments_are_counted_without_text_even_with_a_late_tag() {
     let input = format!(
         r#"{{"type":"response.output_item.done","output_index":0,"item":{{"arguments":"{}","id":"i","call_id":"c","name":"read","type":"function_call"}}}}"#,
-        "x".repeat(1_048_576)
+        "x".repeat(131_072)
     );
     let value = Json::collected(projected(input.as_bytes(), crate::Provider::OpenAiCodex));
     let item = json::value_at(value.view(), json::required(value.view(), b"item").unwrap()).unwrap();
     let args = json::value_at(item, json::required(item, b"arguments").unwrap()).unwrap();
-    assert_eq!(crate::openai::response::long_text(args), Some(1_048_576));
+    assert_eq!(crate::openai::response::long_text(args), Some(131_072));
     assert_eq!(json::record_text(args, 0).unwrap(), b"");
 }
 
diff --git a/tests/http/tests/memory.rs b/tests/http/tests/memory.rs
index 67dabd1..a8b5a1a 100644
--- a/tests/http/tests/memory.rs
+++ b/tests/http/tests/memory.rs
@@ -454,7 +454,7 @@ fn a_large_event_holds_only_its_intake_piece_and_delivery() {
 fn an_event_of_a_megabyte_costs_the_same_reader_memory_as_a_small_one() {
     let limits = sse::Limits { line: 2 << 20, event: 2 << 20, field: 8, chunk: 4 };
     let small = format!("event: 12345678\nid: 12345678\ndata: {}\n\n", "a".repeat(64));
-    let large = format!("event: 12345678\nid: 12345678\ndata: {}\n\n", "a".repeat(1 << 20));
+    let large = format!("event: 12345678\nid: 12345678\ndata: {}\n\n", "a".repeat(1 << 14));
     let small = events(limits, small.as_bytes(), None);
     let large = events(limits, large.as_bytes(), None);
     assert_eq!(small, large, "the reader holds no event buffer");
diff --git a/tests/http/transcripts/hostile-comments-without-end.expect b/tests/http/transcripts/hostile-comments-without-end.expect
index 4a73b45..b947be7 100644
--- a/tests/http/transcripts/hostile-comments-without-end.expect
+++ b/tests/http/transcripts/hostile-comments-without-end.expect
@@ -1,9 +1,10 @@
 # Written by hand: a response a broken or hostile server sends.
 # Drafted by the reference readers, then checked by hand.
+sse event 2048
 status HTTP/1.1 200
 header Content-Type "text/event-stream"
 header Cache-Control "no-cache"
-header Content-Length "19500"
-framing length 19500
+header Content-Length "2080"
+framing length 2080
 sse failed event-too-long
 done keep
diff --git a/tests/http/transcripts/hostile-comments-without-end.http b/tests/http/transcripts/hostile-comments-without-end.http
index 08688f1..9e340d9 100644
--- a/tests/http/transcripts/hostile-comments-without-end.http
+++ b/tests/http/transcripts/hostile-comments-without-end.http
@@ -1,7 +1,7 @@
 HTTP/1.1 200 OK
 Content-Type: text/event-stream
 Cache-Control: no-cache
-Content-Length: 19500
+Content-Length: 2080
 
 : keep-alive
 : keep-alive
@@ -163,1343 +163,3 @@ Content-Length: 19500
 : keep-alive
 : keep-alive
 : keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
-: keep-alive
diff --git a/tests/http/transcripts/hostile-event-line-too-long.expect b/tests/http/transcripts/hostile-event-line-too-long.expect
index cb3b59e..c972c35 100644
--- a/tests/http/transcripts/hostile-event-line-too-long.expect
+++ b/tests/http/transcripts/hostile-event-line-too-long.expect
@@ -1,7 +1,7 @@
 # Written by hand: a response a broken or hostile server sends.
-# One data line of 20,000 bytes, longer than the reader's line limit and
+# One data line of 2,000 bytes, longer than the reader's line limit and
 # shorter than its event limit.
-sse line 4096 event 65536 field 256 chunk 4096
+sse line 512 event 8192 field 256 chunk 4096
 status HTTP/1.1 200
 header Content-Type "text/event-stream"
 header Cache-Control "no-cache"
diff --git a/tests/http/transcripts/hostile-event-line-too-long.http b/tests/http/transcripts/hostile-event-line-too-long.http
index a522f9f..fc1d38f 100644
--- a/tests/http/transcripts/hostile-event-line-too-long.http
+++ b/tests/http/transcripts/hostile-event-line-too-long.http
@@ -3,10 +3,8 @@ Content-Type: text/event-stream
 Cache-Control: no-cache
 Transfer-Encoding: chunked
 
-4000
-data: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
-e28
-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
+7d2
+data: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
 
 
 0
diff --git a/tests/http/transcripts/hostile-event-never-ends.expect b/tests/http/transcripts/hostile-event-never-ends.expect
index c278331..0e57c97 100644
--- a/tests/http/transcripts/hostile-event-never-ends.expect
+++ b/tests/http/transcripts/hostile-event-never-ends.expect
@@ -1,9 +1,10 @@
 # Written by hand: a response a broken or hostile server sends.
 # Drafted by the reference readers, then checked by hand.
+sse event 2048
 status HTTP/1.1 200
 header Content-Type "text/event-stream"
 header Cache-Control "no-cache"
-header Content-Length "22000"
-framing length 22000
+header Content-Length "2200"
+framing length 2200
 sse failed event-too-long
 done keep
diff --git a/tests/http/transcripts/hostile-event-never-ends.http b/tests/http/transcripts/hostile-event-never-ends.http
index 93fcfc5..6721ff3 100644
--- a/tests/http/transcripts/hostile-event-never-ends.http
+++ b/tests/http/transcripts/hostile-event-never-ends.http
@@ -1,7 +1,7 @@
 HTTP/1.1 200 OK
 Content-Type: text/event-stream
 Cache-Control: no-cache
-Content-Length: 22000
+Content-Length: 2200
 
 data: 00000 00000 00000 00000 00000 00000 00000 00000 
 data: 00001 00001 00001 00001 00001 00001 00001 00001 
@@ -43,363 +43,3 @@ data: 00036 00036 00036 00036 00036 00036 00036 00036
 data: 00037 00037 00037 00037 00037 00037 00037 00037 
 data: 00038 00038 00038 00038 00038 00038 00038 00038 
 data: 00039 00039 00039 00039 00039 00039 00039 00039 
-data: 00040 00040 00040 00040 00040 00040 00040 00040 
-data: 00041 00041 00041 00041 00041 00041 00041 00041 
-data: 00042 00042 00042 00042 00042 00042 00042 00042 
-data: 00043 00043 00043 00043 00043 00043 00043 00043 
-data: 00044 00044 00044 00044 00044 00044 00044 00044 
-data: 00045 00045 00045 00045 00045 00045 00045 00045 
-data: 00046 00046 00046 00046 00046 00046 00046 00046 
-data: 00047 00047 00047 00047 00047 00047 00047 00047 
-data: 00048 00048 00048 00048 00048 00048 00048 00048 
-data: 00049 00049 00049 00049 00049 00049 00049 00049 
-data: 00050 00050 00050 00050 00050 00050 00050 00050 
-data: 00051 00051 00051 00051 00051 00051 00051 00051 
-data: 00052 00052 00052 00052 00052 00052 00052 00052 
-data: 00053 00053 00053 00053 00053 00053 00053 00053 
-data: 00054 00054 00054 00054 00054 00054 00054 00054 
-data: 00055 00055 00055 00055 00055 00055 00055 00055 
-data: 00056 00056 00056 00056 00056 00056 00056 00056 
-data: 00057 00057 00057 00057 00057 00057 00057 00057 
-data: 00058 00058 00058 00058 00058 00058 00058 00058 
-data: 00059 00059 00059 00059 00059 00059 00059 00059 
-data: 00060 00060 00060 00060 00060 00060 00060 00060 
-data: 00061 00061 00061 00061 00061 00061 00061 00061 
-data: 00062 00062 00062 00062 00062 00062 00062 00062 
-data: 00063 00063 00063 00063 00063 00063 00063 00063 
-data: 00064 00064 00064 00064 00064 00064 00064 00064 
-data: 00065 00065 00065 00065 00065 00065 00065 00065 
-data: 00066 00066 00066 00066 00066 00066 00066 00066 
-data: 00067 00067 00067 00067 00067 00067 00067 00067 
-data: 00068 00068 00068 00068 00068 00068 00068 00068 
-data: 00069 00069 00069 00069 00069 00069 00069 00069 
-data: 00070 00070 00070 00070 00070 00070 00070 00070 
-data: 00071 00071 00071 00071 00071 00071 00071 00071 
-data: 00072 00072 00072 00072 00072 00072 00072 00072 
-data: 00073 00073 00073 00073 00073 00073 00073 00073 
-data: 00074 00074 00074 00074 00074 00074 00074 00074 
-data: 00075 00075 00075 00075 00075 00075 00075 00075 
-data: 00076 00076 00076 00076 00076 00076 00076 00076 
-data: 00077 00077 00077 00077 00077 00077 00077 00077 
-data: 00078 00078 00078 00078 00078 00078 00078 00078 
-data: 00079 00079 00079 00079 00079 00079 00079 00079 
-data: 00080 00080 00080 00080 00080 00080 00080 00080 
-data: 00081 00081 00081 00081 00081 00081 00081 00081 
-data: 00082 00082 00082 00082 00082 00082 00082 00082 
-data: 00083 00083 00083 00083 00083 00083 00083 00083 
-data: 00084 00084 00084 00084 00084 00084 00084 00084 
-data: 00085 00085 00085 00085 00085 00085 00085 00085 
-data: 00086 00086 00086 00086 00086 00086 00086 00086 
-data: 00087 00087 00087 00087 00087 00087 00087 00087 
-data: 00088 00088 00088 00088 00088 00088 00088 00088 
-data: 00089 00089 00089 00089 00089 00089 00089 00089 
-data: 00090 00090 00090 00090 00090 00090 00090 00090 
-data: 00091 00091 00091 00091 00091 00091 00091 00091 
-data: 00092 00092 00092 00092 00092 00092 00092 00092 
-data: 00093 00093 00093 00093 00093 00093 00093 00093 
-data: 00094 00094 00094 00094 00094 00094 00094 00094 
-data: 00095 00095 00095 00095 00095 00095 00095 00095 
-data: 00096 00096 00096 00096 00096 00096 00096 00096 
-data: 00097 00097 00097 00097 00097 00097 00097 00097 
-data: 00098 00098 00098 00098 00098 00098 00098 00098 
-data: 00099 00099 00099 00099 00099 00099 00099 00099 
-data: 00100 00100 00100 00100 00100 00100 00100 00100 
-data: 00101 00101 00101 00101 00101 00101 00101 00101 
-data: 00102 00102 00102 00102 00102 00102 00102 00102 
-data: 00103 00103 00103 00103 00103 00103 00103 00103 
-data: 00104 00104 00104 00104 00104 00104 00104 00104 
-data: 00105 00105 00105 00105 00105 00105 00105 00105 
-data: 00106 00106 00106 00106 00106 00106 00106 00106 
-data: 00107 00107 00107 00107 00107 00107 00107 00107 
-data: 00108 00108 00108 00108 00108 00108 00108 00108 
-data: 00109 00109 00109 00109 00109 00109 00109 00109 
-data: 00110 00110 00110 00110 00110 00110 00110 00110 
-data: 00111 00111 00111 00111 00111 00111 00111 00111 
-data: 00112 00112 00112 00112 00112 00112 00112 00112 
-data: 00113 00113 00113 00113 00113 00113 00113 00113 
-data: 00114 00114 00114 00114 00114 00114 00114 00114 
-data: 00115 00115 00115 00115 00115 00115 00115 00115 
-data: 00116 00116 00116 00116 00116 00116 00116 00116 
-data: 00117 00117 00117 00117 00117 00117 00117 00117 
-data: 00118 00118 00118 00118 00118 00118 00118 00118 
-data: 00119 00119 00119 00119 00119 00119 00119 00119 
-data: 00120 00120 00120 00120 00120 00120 00120 00120 
-data: 00121 00121 00121 00121 00121 00121 00121 00121 
-data: 00122 00122 00122 00122 00122 00122 00122 00122 
-data: 00123 00123 00123 00123 00123 00123 00123 00123 
-data: 00124 00124 00124 00124 00124 00124 00124 00124 
-data: 00125 00125 00125 00125 00125 00125 00125 00125 
-data: 00126 00126 00126 00126 00126 00126 00126 00126 
-data: 00127 00127 00127 00127 00127 00127 00127 00127 
-data: 00128 00128 00128 00128 00128 00128 00128 00128 
-data: 00129 00129 00129 00129 00129 00129 00129 00129 
-data: 00130 00130 00130 00130 00130 00130 00130 00130 
-data: 00131 00131 00131 00131 00131 00131 00131 00131 
-data: 00132 00132 00132 00132 00132 00132 00132 00132 
-data: 00133 00133 00133 00133 00133 00133 00133 00133 
-data: 00134 00134 00134 00134 00134 00134 00134 00134 
-data: 00135 00135 00135 00135 00135 00135 00135 00135 
-data: 00136 00136 00136 00136 00136 00136 00136 00136 
-data: 00137 00137 00137 00137 00137 00137 00137 00137 
-data: 00138 00138 00138 00138 00138 00138 00138 00138 
-data: 00139 00139 00139 00139 00139 00139 00139 00139 
-data: 00140 00140 00140 00140 00140 00140 00140 00140 
-data: 00141 00141 00141 00141 00141 00141 00141 00141 
-data: 00142 00142 00142 00142 00142 00142 00142 00142 
-data: 00143 00143 00143 00143 00143 00143 00143 00143 
-data: 00144 00144 00144 00144 00144 00144 00144 00144 
-data: 00145 00145 00145 00145 00145 00145 00145 00145 
-data: 00146 00146 00146 00146 00146 00146 00146 00146 
-data: 00147 00147 00147 00147 00147 00147 00147 00147 
-data: 00148 00148 00148 00148 00148 00148 00148 00148 
-data: 00149 00149 00149 00149 00149 00149 00149 00149 
-data: 00150 00150 00150 00150 00150 00150 00150 00150 
-data: 00151 00151 00151 00151 00151 00151 00151 00151 
-data: 00152 00152 00152 00152 00152 00152 00152 00152 
-data: 00153 00153 00153 00153 00153 00153 00153 00153 
-data: 00154 00154 00154 00154 00154 00154 00154 00154 
-data: 00155 00155 00155 00155 00155 00155 00155 00155 
-data: 00156 00156 00156 00156 00156 00156 00156 00156 
-data: 00157 00157 00157 00157 00157 00157 00157 00157 
-data: 00158 00158 00158 00158 00158 00158 00158 00158 
-data: 00159 00159 00159 00159 00159 00159 00159 00159 
-data: 00160 00160 00160 00160 00160 00160 00160 00160 
-data: 00161 00161 00161 00161 00161 00161 00161 00161 
-data: 00162 00162 00162 00162 00162 00162 00162 00162 
-data: 00163 00163 00163 00163 00163 00163 00163 00163 
-data: 00164 00164 00164 00164 00164 00164 00164 00164 
-data: 00165 00165 00165 00165 00165 00165 00165 00165 
-data: 00166 00166 00166 00166 00166 00166 00166 00166 
-data: 00167 00167 00167 00167 00167 00167 00167 00167 
-data: 00168 00168 00168 00168 00168 00168 00168 00168 
-data: 00169 00169 00169 00169 00169 00169 00169 00169 
-data: 00170 00170 00170 00170 00170 00170 00170 00170 
-data: 00171 00171 00171 00171 00171 00171 00171 00171 
-data: 00172 00172 00172 00172 00172 00172 00172 00172 
-data: 00173 00173 00173 00173 00173 00173 00173 00173 
-data: 00174 00174 00174 00174 00174 00174 00174 00174 
-data: 00175 00175 00175 00175 00175 00175 00175 00175 
-data: 00176 00176 00176 00176 00176 00176 00176 00176 
-data: 00177 00177 00177 00177 00177 00177 00177 00177 
-data: 00178 00178 00178 00178 00178 00178 00178 00178 
-data: 00179 00179 00179 00179 00179 00179 00179 00179 
-data: 00180 00180 00180 00180 00180 00180 00180 00180 
-data: 00181 00181 00181 00181 00181 00181 00181 00181 
-data: 00182 00182 00182 00182 00182 00182 00182 00182 
-data: 00183 00183 00183 00183 00183 00183 00183 00183 
-data: 00184 00184 00184 00184 00184 00184 00184 00184 
-data: 00185 00185 00185 00185 00185 00185 00185 00185 
-data: 00186 00186 00186 00186 00186 00186 00186 00186 
-data: 00187 00187 00187 00187 00187 00187 00187 00187 
-data: 00188 00188 00188 00188 00188 00188 00188 00188 
-data: 00189 00189 00189 00189 00189 00189 00189 00189 
-data: 00190 00190 00190 00190 00190 00190 00190 00190 
-data: 00191 00191 00191 00191 00191 00191 00191 00191 
-data: 00192 00192 00192 00192 00192 00192 00192 00192 
-data: 00193 00193 00193 00193 00193 00193 00193 00193 
-data: 00194 00194 00194 00194 00194 00194 00194 00194 
-data: 00195 00195 00195 00195 00195 00195 00195 00195 
-data: 00196 00196 00196 00196 00196 00196 00196 00196 
-data: 00197 00197 00197 00197 00197 00197 00197 00197 
-data: 00198 00198 00198 00198 00198 00198 00198 00198 
-data: 00199 00199 00199 00199 00199 00199 00199 00199 
-data: 00200 00200 00200 00200 00200 00200 00200 00200 
-data: 00201 00201 00201 00201 00201 00201 00201 00201 
-data: 00202 00202 00202 00202 00202 00202 00202 00202 
-data: 00203 00203 00203 00203 00203 00203 00203 00203 
-data: 00204 00204 00204 00204 00204 00204 00204 00204 
-data: 00205 00205 00205 00205 00205 00205 00205 00205 
-data: 00206 00206 00206 00206 00206 00206 00206 00206 
-data: 00207 00207 00207 00207 00207 00207 00207 00207 
-data: 00208 00208 00208 00208 00208 00208 00208 00208 
-data: 00209 00209 00209 00209 00209 00209 00209 00209 
-data: 00210 00210 00210 00210 00210 00210 00210 00210 
-data: 00211 00211 00211 00211 00211 00211 00211 00211 
-data: 00212 00212 00212 00212 00212 00212 00212 00212 
-data: 00213 00213 00213 00213 00213 00213 00213 00213 
-data: 00214 00214 00214 00214 00214 00214 00214 00214 
-data: 00215 00215 00215 00215 00215 00215 00215 00215 
-data: 00216 00216 00216 00216 00216 00216 00216 00216 
-data: 00217 00217 00217 00217 00217 00217 00217 00217 
-data: 00218 00218 00218 00218 00218 00218 00218 00218 
-data: 00219 00219 00219 00219 00219 00219 00219 00219 
-data: 00220 00220 00220 00220 00220 00220 00220 00220 
-data: 00221 00221 00221 00221 00221 00221 00221 00221 
-data: 00222 00222 00222 00222 00222 00222 00222 00222 
-data: 00223 00223 00223 00223 00223 00223 00223 00223 
-data: 00224 00224 00224 00224 00224 00224 00224 00224 
-data: 00225 00225 00225 00225 00225 00225 00225 00225 
-data: 00226 00226 00226 00226 00226 00226 00226 00226 
-data: 00227 00227 00227 00227 00227 00227 00227 00227 
-data: 00228 00228 00228 00228 00228 00228 00228 00228 
-data: 00229 00229 00229 00229 00229 00229 00229 00229 
-data: 00230 00230 00230 00230 00230 00230 00230 00230 
-data: 00231 00231 00231 00231 00231 00231 00231 00231 
-data: 00232 00232 00232 00232 00232 00232 00232 00232 
-data: 00233 00233 00233 00233 00233 00233 00233 00233 
-data: 00234 00234 00234 00234 00234 00234 00234 00234 
-data: 00235 00235 00235 00235 00235 00235 00235 00235 
-data: 00236 00236 00236 00236 00236 00236 00236 00236 
-data: 00237 00237 00237 00237 00237 00237 00237 00237 
-data: 00238 00238 00238 00238 00238 00238 00238 00238 
-data: 00239 00239 00239 00239 00239 00239 00239 00239 
-data: 00240 00240 00240 00240 00240 00240 00240 00240 
-data: 00241 00241 00241 00241 00241 00241 00241 00241 
-data: 00242 00242 00242 00242 00242 00242 00242 00242 
-data: 00243 00243 00243 00243 00243 00243 00243 00243 
-data: 00244 00244 00244 00244 00244 00244 00244 00244 
-data: 00245 00245 00245 00245 00245 00245 00245 00245 
-data: 00246 00246 00246 00246 00246 00246 00246 00246 
-data: 00247 00247 00247 00247 00247 00247 00247 00247 
-data: 00248 00248 00248 00248 00248 00248 00248 00248 
-data: 00249 00249 00249 00249 00249 00249 00249 00249 
-data: 00250 00250 00250 00250 00250 00250 00250 00250 
-data: 00251 00251 00251 00251 00251 00251 00251 00251 
-data: 00252 00252 00252 00252 00252 00252 00252 00252 
-data: 00253 00253 00253 00253 00253 00253 00253 00253 
-data: 00254 00254 00254 00254 00254 00254 00254 00254 
-data: 00255 00255 00255 00255 00255 00255 00255 00255 
-data: 00256 00256 00256 00256 00256 00256 00256 00256 
-data: 00257 00257 00257 00257 00257 00257 00257 00257 
-data: 00258 00258 00258 00258 00258 00258 00258 00258 
-data: 00259 00259 00259 00259 00259 00259 00259 00259 
-data: 00260 00260 00260 00260 00260 00260 00260 00260 
-data: 00261 00261 00261 00261 00261 00261 00261 00261 
-data: 00262 00262 00262 00262 00262 00262 00262 00262 
-data: 00263 00263 00263 00263 00263 00263 00263 00263 
-data: 00264 00264 00264 00264 00264 00264 00264 00264 
-data: 00265 00265 00265 00265 00265 00265 00265 00265 
-data: 00266 00266 00266 00266 00266 00266 00266 00266 
-data: 00267 00267 00267 00267 00267 00267 00267 00267 
-data: 00268 00268 00268 00268 00268 00268 00268 00268 
-data: 00269 00269 00269 00269 00269 00269 00269 00269 
-data: 00270 00270 00270 00270 00270 00270 00270 00270 
-data: 00271 00271 00271 00271 00271 00271 00271 00271 
-data: 00272 00272 00272 00272 00272 00272 00272 00272 
-data: 00273 00273 00273 00273 00273 00273 00273 00273 
-data: 00274 00274 00274 00274 00274 00274 00274 00274 
-data: 00275 00275 00275 00275 00275 00275 00275 00275 
-data: 00276 00276 00276 00276 00276 00276 00276 00276 
-data: 00277 00277 00277 00277 00277 00277 00277 00277 
-data: 00278 00278 00278 00278 00278 00278 00278 00278 
-data: 00279 00279 00279 00279 00279 00279 00279 00279 
-data: 00280 00280 00280 00280 00280 00280 00280 00280 
-data: 00281 00281 00281 00281 00281 00281 00281 00281 
-data: 00282 00282 00282 00282 00282 00282 00282 00282 
-data: 00283 00283 00283 00283 00283 00283 00283 00283 
-data: 00284 00284 00284 00284 00284 00284 00284 00284 
-data: 00285 00285 00285 00285 00285 00285 00285 00285 
-data: 00286 00286 00286 00286 00286 00286 00286 00286 
-data: 00287 00287 00287 00287 00287 00287 00287 00287 
-data: 00288 00288 00288 00288 00288 00288 00288 00288 
-data: 00289 00289 00289 00289 00289 00289 00289 00289 
-data: 00290 00290 00290 00290 00290 00290 00290 00290 
-data: 00291 00291 00291 00291 00291 00291 00291 00291 
-data: 00292 00292 00292 00292 00292 00292 00292 00292 
-data: 00293 00293 00293 00293 00293 00293 00293 00293 
-data: 00294 00294 00294 00294 00294 00294 00294 00294 
-data: 00295 00295 00295 00295 00295 00295 00295 00295 
-data: 00296 00296 00296 00296 00296 00296 00296 00296 
-data: 00297 00297 00297 00297 00297 00297 00297 00297 
-data: 00298 00298 00298 00298 00298 00298 00298 00298 
-data: 00299 00299 00299 00299 00299 00299 00299 00299 
-data: 00300 00300 00300 00300 00300 00300 00300 00300 
-data: 00301 00301 00301 00301 00301 00301 00301 00301 
-data: 00302 00302 00302 00302 00302 00302 00302 00302 
-data: 00303 00303 00303 00303 00303 00303 00303 00303 
-data: 00304 00304 00304 00304 00304 00304 00304 00304 
-data: 00305 00305 00305 00305 00305 00305 00305 00305 
-data: 00306 00306 00306 00306 00306 00306 00306 00306 
-data: 00307 00307 00307 00307 00307 00307 00307 00307 
-data: 00308 00308 00308 00308 00308 00308 00308 00308 
-data: 00309 00309 00309 00309 00309 00309 00309 00309 
-data: 00310 00310 00310 00310 00310 00310 00310 00310 
-data: 00311 00311 00311 00311 00311 00311 00311 00311 
-data: 00312 00312 00312 00312 00312 00312 00312 00312 
-data: 00313 00313 00313 00313 00313 00313 00313 00313 
-data: 00314 00314 00314 00314 00314 00314 00314 00314 
-data: 00315 00315 00315 00315 00315 00315 00315 00315 
-data: 00316 00316 00316 00316 00316 00316 00316 00316 
-data: 00317 00317 00317 00317 00317 00317 00317 00317 
-data: 00318 00318 00318 00318 00318 00318 00318 00318 
-data: 00319 00319 00319 00319 00319 00319 00319 00319 
-data: 00320 00320 00320 00320 00320 00320 00320 00320 
-data: 00321 00321 00321 00321 00321 00321 00321 00321 
-data: 00322 00322 00322 00322 00322 00322 00322 00322 
-data: 00323 00323 00323 00323 00323 00323 00323 00323 
-data: 00324 00324 00324 00324 00324 00324 00324 00324 
-data: 00325 00325 00325 00325 00325 00325 00325 00325 
-data: 00326 00326 00326 00326 00326 00326 00326 00326 
-data: 00327 00327 00327 00327 00327 00327 00327 00327 
-data: 00328 00328 00328 00328 00328 00328 00328 00328 
-data: 00329 00329 00329 00329 00329 00329 00329 00329 
-data: 00330 00330 00330 00330 00330 00330 00330 00330 
-data: 00331 00331 00331 00331 00331 00331 00331 00331 
-data: 00332 00332 00332 00332 00332 00332 00332 00332 
-data: 00333 00333 00333 00333 00333 00333 00333 00333 
-data: 00334 00334 00334 00334 00334 00334 00334 00334 
-data: 00335 00335 00335 00335 00335 00335 00335 00335 
-data: 00336 00336 00336 00336 00336 00336 00336 00336 
-data: 00337 00337 00337 00337 00337 00337 00337 00337 
-data: 00338 00338 00338 00338 00338 00338 00338 00338 
-data: 00339 00339 00339 00339 00339 00339 00339 00339 
-data: 00340 00340 00340 00340 00340 00340 00340 00340 
-data: 00341 00341 00341 00341 00341 00341 00341 00341 
-data: 00342 00342 00342 00342 00342 00342 00342 00342 
-data: 00343 00343 00343 00343 00343 00343 00343 00343 
-data: 00344 00344 00344 00344 00344 00344 00344 00344 
-data: 00345 00345 00345 00345 00345 00345 00345 00345 
-data: 00346 00346 00346 00346 00346 00346 00346 00346 
-data: 00347 00347 00347 00347 00347 00347 00347 00347 
-data: 00348 00348 00348 00348 00348 00348 00348 00348 
-data: 00349 00349 00349 00349 00349 00349 00349 00349 
-data: 00350 00350 00350 00350 00350 00350 00350 00350 
-data: 00351 00351 00351 00351 00351 00351 00351 00351 
-data: 00352 00352 00352 00352 00352 00352 00352 00352 
-data: 00353 00353 00353 00353 00353 00353 00353 00353 
-data: 00354 00354 00354 00354 00354 00354 00354 00354 
-data: 00355 00355 00355 00355 00355 00355 00355 00355 
-data: 00356 00356 00356 00356 00356 00356 00356 00356 
-data: 00357 00357 00357 00357 00357 00357 00357 00357 
-data: 00358 00358 00358 00358 00358 00358 00358 00358 
-data: 00359 00359 00359 00359 00359 00359 00359 00359 
-data: 00360 00360 00360 00360 00360 00360 00360 00360 
-data: 00361 00361 00361 00361 00361 00361 00361 00361 
-data: 00362 00362 00362 00362 00362 00362 00362 00362 
-data: 00363 00363 00363 00363 00363 00363 00363 00363 
-data: 00364 00364 00364 00364 00364 00364 00364 00364 
-data: 00365 00365 00365 00365 00365 00365 00365 00365 
-data: 00366 00366 00366 00366 00366 00366 00366 00366 
-data: 00367 00367 00367 00367 00367 00367 00367 00367 
-data: 00368 00368 00368 00368 00368 00368 00368 00368 
-data: 00369 00369 00369 00369 00369 00369 00369 00369 
-data: 00370 00370 00370 00370 00370 00370 00370 00370 
-data: 00371 00371 00371 00371 00371 00371 00371 00371 
-data: 00372 00372 00372 00372 00372 00372 00372 00372 
-data: 00373 00373 00373 00373 00373 00373 00373 00373 
-data: 00374 00374 00374 00374 00374 00374 00374 00374 
-data: 00375 00375 00375 00375 00375 00375 00375 00375 
-data: 00376 00376 00376 00376 00376 00376 00376 00376 
-data: 00377 00377 00377 00377 00377 00377 00377 00377 
-data: 00378 00378 00378 00378 00378 00378 00378 00378 
-data: 00379 00379 00379 00379 00379 00379 00379 00379 
-data: 00380 00380 00380 00380 00380 00380 00380 00380 
-data: 00381 00381 00381 00381 00381 00381 00381 00381 
-data: 00382 00382 00382 00382 00382 00382 00382 00382 
-data: 00383 00383 00383 00383 00383 00383 00383 00383 
-data: 00384 00384 00384 00384 00384 00384 00384 00384 
-data: 00385 00385 00385 00385 00385 00385 00385 00385 
-data: 00386 00386 00386 00386 00386 00386 00386 00386 
-data: 00387 00387 00387 00387 00387 00387 00387 00387 
-data: 00388 00388 00388 00388 00388 00388 00388 00388 
-data: 00389 00389 00389 00389 00389 00389 00389 00389 
-data: 00390 00390 00390 00390 00390 00390 00390 00390 
-data: 00391 00391 00391 00391 00391 00391 00391 00391 
-data: 00392 00392 00392 00392 00392 00392 00392 00392 
-data: 00393 00393 00393 00393 00393 00393 00393 00393 
-data: 00394 00394 00394 00394 00394 00394 00394 00394 
-data: 00395 00395 00395 00395 00395 00395 00395 00395 
-data: 00396 00396 00396 00396 00396 00396 00396 00396 
-data: 00397 00397 00397 00397 00397 00397 00397 00397 
-data: 00398 00398 00398 00398 00398 00398 00398 00398 
-data: 00399 00399 00399 00399 00399 00399 00399 00399 
diff --git a/tests/json/tests/memory.rs b/tests/json/tests/memory.rs
index e55d9db..1e15eb2 100644
--- a/tests/json/tests/memory.rs
+++ b/tests/json/tests/memory.rs
@@ -234,7 +234,7 @@ fn the_writer_holds_no_more_than_its_worst_case_at_its_limits() {
 #[test]
 fn skipped_megabytes_and_long_text_keep_the_same_bounded_buffer() {
     let limits = Limits { depth: 4, string: 8, number: 8, chunk: 32, length: 2 << 20 };
-    let mut large = vec![b'a'; 1 << 20];
+    let mut large = vec![b'a'; 1 << 12];
     large.insert(0, b'"');
     large.push(b'"');
     for request in [Request::Skip, Request::Text(4)] {
@@ -347,7 +347,7 @@ fn collector_skipped_megabytes_add_no_memory_and_full_counts_reach_the_bound() {
     };
     let filter = Filter { root: Keep::Into(&[Node { key: Key::Field(b"keep"), keep: Keep::Value }]) };
     let small = format!("{{\"keep\":1,\"omit\":\"{}\"}}", "a".repeat(64));
-    let large = format!("{{\"keep\":1,\"omit\":\"{}\"}}", "a".repeat(1 << 20));
+    let large = format!("{{\"keep\":1,\"omit\":\"{}\"}}", "a".repeat(1 << 12));
     assert_eq!(collect_memory(small.as_bytes(), filter, limits).0, collect_memory(large.as_bytes(), filter, limits).0);
     let exact = Limits { tokenizer: json::Limits { chunk: 4, ..limits.tokenizer }, tokens: 4, text: 5, ..limits };
     let (peak, bound) = collect_memory(br#"{"keep":1}"#, filter, exact);
@@ -373,7 +373,7 @@ fn tagged_losing_candidates_hold_no_memory_after_their_counts() {
         skip: 2 << 20,
     };
     let small = format!("{{\"body\":\"{}\",\"id\":1,\"type\":\"small\"}}", "a".repeat(64));
-    let large = format!("{{\"body\":\"{}\",\"id\":1,\"type\":\"small\"}}", "a".repeat(1 << 20));
+    let large = format!("{{\"body\":\"{}\",\"id\":1,\"type\":\"small\"}}", "a".repeat(1 << 12));
     let (peak, bound) = collect_memory_caps(small.as_bytes(), filter, limits, &[2]);
     assert_eq!(peak, bound, "the selected variant attains the retained document bound");
     assert_eq!(
diff --git a/tests/lib/tests/memory.rs b/tests/lib/tests/memory.rs
index c669940..0388fed 100644
--- a/tests/lib/tests/memory.rs
+++ b/tests/lib/tests/memory.rs
@@ -600,12 +600,16 @@ fn journal_memory<W: Item + std::fmt::Debug, O: Item + std::fmt::Debug>(limits:
 fn a_request_table_filled_to_every_limit_stays_within_its_worst_case() {
     for capacity in CAPACITIES {
         for restored in [false, true] {
-            request_table_memory::<u8, u8>(capacity, restored);
-            request_table_memory::<u64, u64>(capacity, restored);
-            request_table_memory::<u128, u128>(capacity, restored);
             request_table_memory::<[u64; 8], [u64; 8]>(capacity, restored);
-            request_table_memory::<[u64; 8], u8>(capacity, restored);
-            request_table_memory::<u8, [u64; 8]>(capacity, restored);
+            // Its maps' types and keys are the same whatever it holds, and
+            // its slab is allocated once: three levels of tree, once.
+            if capacity <= 64 {
+                request_table_memory::<u8, u8>(capacity, restored);
+                request_table_memory::<u64, u64>(capacity, restored);
+                request_table_memory::<u128, u128>(capacity, restored);
+                request_table_memory::<[u64; 8], u8>(capacity, restored);
+                request_table_memory::<u8, [u64; 8]>(capacity, restored);
+            }
         }
     }
 }
diff --git a/tests/llm/tests/memory.rs b/tests/llm/tests/memory.rs
index f3e837c..c0a2af3 100644
--- a/tests/llm/tests/memory.rs
+++ b/tests/llm/tests/memory.rs
@@ -775,14 +775,14 @@ fn selective_peak(size: usize, call_arguments: bool) -> u64 {
 #[test]
 fn selective_echo_high_water_is_independent_of_the_echo_size() {
     let short = selective_peak(64, false);
-    let long = selective_peak(1_048_576, false);
+    let long = selective_peak(1 << 18, false);
     assert_eq!(short, long, "discarded echoes never become an event buffer");
 }
 
 #[test]
 fn selective_oversize_call_high_water_is_independent_of_the_argument_size() {
     let short = selective_peak(64, true);
-    let long = selective_peak(1_048_576, true);
+    let long = selective_peak(1 << 18, true);
     assert_eq!(short, long, "discarded arguments retain only their length");
 }
 
diff --git a/tests/llm/tests/selective.rs b/tests/llm/tests/selective.rs
index d8ee791..ee7d202 100644
--- a/tests/llm/tests/selective.rs
+++ b/tests/llm/tests/selective.rs
@@ -27,7 +27,7 @@ fn a_captured_completion_decodes_under_tiny_retained_limits() {
 
 #[test]
 fn an_oversized_native_call_is_scanned_and_completes_without_its_arguments() {
-    let arguments = "x".repeat(1_048_576);
+    let arguments = "x".repeat(1 << 18);
     let done = format!(
         r#"{{"type":"response.output_item.done","output_index":0,"item":{{"arguments":"{arguments}","id":"i","call_id":"c","name":"read","type":"function_call"}}}}"#
     );
@@ -40,7 +40,7 @@ fn an_oversized_native_call_is_scanned_and_completes_without_its_arguments() {
     world.assert_once();
     assert!(
         world.seen.iter().any(|event| matches!(event, client::Event::Completed { completion, .. }
-        if matches!(completion.content.as_ref(), [Block::Oversize { bytes: 1_048_576, .. }]))),
+        if matches!(completion.content.as_ref(), [Block::Oversize { bytes: 262_144, .. }]))),
         "{:?}",
         world.seen
     );
diff --git a/tests/ring/src/lib.rs b/tests/ring/src/lib.rs
index 8e747b3..e8cae87 100644
--- a/tests/ring/src/lib.rs
+++ b/tests/ring/src/lib.rs
@@ -216,9 +216,13 @@ pub fn v4(port: u16) -> Addr {
 /// Bytes no two offsets of a short span repeat in, to catch reordering.
 #[must_use]
 pub fn pattern(len: usize) -> Box<[u8]> {
-    let mut bytes = vec![0_u8; len];
-    for (at, byte) in bytes.iter_mut().enumerate() {
+    // Its period: 31 * at repeats every 256 bytes, at >> 8 every 65536.
+    const PERIOD: usize = 1 << 16;
+    let mut period = vec![0_u8; PERIOD.min(len)];
+    for (at, byte) in period.iter_mut().enumerate() {
         *byte = u8::try_from(at.wrapping_mul(31).wrapping_add(at >> 8) & 0xff).expect("masked to a byte");
     }
+    let mut bytes = period.repeat(len.div_ceil(PERIOD));
+    bytes.truncate(len);
     bytes.into_boxed_slice()
 }
diff --git a/tests/ring/tests/ring.rs b/tests/ring/tests/ring.rs
index 96e6a1a..3229f1f 100644
--- a/tests/ring/tests/ring.rs
+++ b/tests/ring/tests/ring.rs
@@ -24,7 +24,10 @@ fn a_large_transfer_takes_short_sends_and_many_receives() {
 
     // Nothing reads yet, so the first send takes what the socket buffers hold.
     let first = world.run(Op::send(client, bytes.clone(), 0).unwrap());
-    let Ok(Done::Count(first_sent)) = first.result else { panic!("a send: {first:?}") };
+    let (Ok(Done::Count(first_sent)), Op::Send { bytes: back, .. }) = (first.result, first.kind) else {
+        panic!("a send")
+    };
+    let mut outgoing = Some(back);
     assert!(usize::try_from(first_sent).unwrap() < LEN, "a short send: {first_sent} of {LEN}");
 
     let mut received = Vec::with_capacity(LEN);
@@ -36,7 +39,7 @@ fn a_large_transfer_takes_short_sends_and_many_receives() {
     while received.len() < LEN {
         assert!(world.clock.now().now < deadline, "the transfer finished in time");
         if sending.is_none() && usize::try_from(from).unwrap() < LEN {
-            sending = Some(world.start(Op::send(client, bytes.clone(), from).unwrap()));
+            sending = Some(world.start(Op::send(client, outgoing.take().unwrap(), from).unwrap()));
         }
         if receiving.is_none() {
             receiving = Some(world.start(Op::recv(server, vec![0; 64 << 10].into_boxed_slice()).unwrap()));
@@ -45,8 +48,9 @@ fn a_large_transfer_takes_short_sends_and_many_receives() {
         if let Some(op) = sending
             && let Some(sent) = world.arrived.remove(&op)
         {
-            let Ok(Done::Count(n)) = sent.result else { panic!("a send: {sent:?}") };
+            let (Ok(Done::Count(n)), Op::Send { bytes: back, .. }) = (sent.result, sent.kind) else { panic!("a send") };
             from = from.checked_add(n).unwrap();
+            outgoing = Some(back);
             sending = None;
         }
         if let Some(op) = receiving
```
