# The read of what has arrived: the code it touches

A read-only audit by the planner's sub-agent, 2026-10-13, for skein 01, smith 01 and jig 05. Lines are at skein `2068257`, smith `6449a197` and jig `d91a89c45`; smith's sessions cite smith `0305f2fd`'s where they moved. Sections: 1, every side below that meets a `Read`; 2, `Intake`; 3, io's path; 4, the readers of unframed bytes, and those that stay exact; 5, every exhaustive match; 6, the design text; 7, how the simulator cuts arrivals.

Line numbers at skein 2068257, smith 6449a197, jig d91a89c45.

## 1. Sides below that meet a Read
a. Through Intake::meet (pick up a new variant once Intake handles it):
- io kernel socket skein-io/src/stream.rs: delivered :972-1011 calls intake.meet :986; exhaustive :999-1003 (End rule), :1017-1037 (demand asserts), :1038-1041 (outstanding).
- io joined end skein-io/src/joined/end.rs: request :66-105, asserts :77-85; delivery :245-255 via stream::delivered; peer moves :304-315.
- io pipe skein-io/src/pipe.rs: request :131-147 compares read == Nothing only (not exhaustive); progress :328-341 own meet + End, does NOT share delivered.
- TLS client skein-tls/src/client/: above.rs:142-163 answer (meet :147), native.rs:537; exhaustive above.rs:166-171 largest (used above.rs:60, native.rs:415), close.rs:97-100. Below reads Fill(5) then Fill(len) per record (below.rs:137, native.rs:595-603); reads below only while a read from above is unmet (below.rs:123-136, native.rs:586-591).
- HTTP body skein-http/src/body.rs: pump :119-140 meets :124; exhaustive piece :172-200 (passes above's read down in same shape), answers :282-295; callers client.rs:844-848, server.rs:1032-1036.
- SSE data skein-http/src/sse.rs: data_demand :360-388 exhaustive :375-384; meet :454-467 (intake.meet :456), cap limits.chunk (:237).
- Tokenizer over held doc via Intake: smith-local-protocol/src/borrowed.rs:155-180; jig-protocol-tools/src/json.rs:29-48; jig-protocol-config/src/document.rs:22-39.
b. Own code, exhaustive (build errors): skein-lib/src/stream/held.rs:35-72 Held::answer (:40-64) (also skein_http::sse::Data, sse.rs:61); tokenizer drivers answering from a slice: skein-json/src/tokenizer.rs:144-155, skein-oauth/src/json.rs:46-56, skein-llm/src/openai/json.rs:67-77, smith-events/src/read.rs:180-190, smith-protocol-machine/src/search.rs:290-300.
c. Kits: skein-world/src/stream.rs:56-119 (Wire, meet :94); skein-fake-peers/src/transport.rs:169-237 (:218). Test sides below with Intake: skein tests/http (client_world.rs:551,653; server_world.rs:784,924,943; sse_world.rs:326,382), tls/src/world.rs:331,432, json/src/world.rs:362,422,689,739, protocol/src/wire.rs:173,209, llm (fake.rs:716,732; lib.rs:263; tests live.rs:222, fake.rs:1029), llm-connection (src/world.rs:240; tests/fake_protocol.rs:210), memory tests (http/tests/memory.rs:230,356; memory_server.rs:247; stack.rs:214; json/tests/memory.rs:73,313); smith tests/protocol-llm/src/translate.rs:287; jig tests/worker-link/src/joined.rs:56, net.rs:110, engine-workers/src/net.rs:112, ops/src/protocol/json.rs:31. Held: skein tests/browser/tests/page_server.rs:25,41.

## 2. Intake (skein-lib/src/intake.rs)
with_capacity :60, worst_case :72, capacity :77, len :83, is_empty :90, room :96, clear :101, append :108; meet :128-145 exhaustive :129-140 (Fill :131-137 asserts fits cap; Scan/Line via scan :168-196 then take); ends_partway :153-164 (only user body.rs:184); take :213-224 copies exactly n bytes, drains, lowers search memo. New arm beside Fill: None when empty, else take(len.min(max)).
Tests: skein-lib/src/tests/streams.rs (cases :48-235, reference :239-262, check :275, split tests :336, :353, :360); fuzzy harness tests/lib/src/intake.rs (reference :18-43, check :52-95), driven by tests/lib/tests/fuzzy_intake.rs:9,41 (cuts seeded :34); Held tests skein-lib/src/tests/held.rs, skein-http/src/tests/data.rs.

## 3. io path (skein-io/src/)
Dispatch layer.rs:677-679. Socket: stream.rs:1619 -> state_request :431-453 -> demand :1015-1051 (asserts, stores, marks ready :1050); Ready::mark -> next (layer.rs:332-334); promote at reclaim (layer.rs:282-287 -> :363-370); resume layer.rs:377-400 pops now :383, stream::resume :394 -> state_resume :371-396 -> follow :592-613 -> deliver :941-969 -> delivered :972-1011. Also met after a receive lands: landed -> follow stream.rs:365-366.
Pipe: pipe.rs:131-147 stores (:145) marks ready (:146); resume layer.rs:395 -> pipe::resume :101-117 -> progress meets :328-334, End :335-341; PipeRead lands: append :187-200, progress :254.
Joined: end.rs:104 -> layer.rs:386-389 -> joined.rs:138-145 -> resume_end :202 -> stream::delivered end.rs:247.
Receive size: intake.room().min(limits.receive) stream.rs:922,964, pipe.rs:344.
Limits::largest_read limits.rs:50-58 = intake; asserted stream.rs:1019-1036, joined/end.rs:77-85; pipes only via Intake::meet. Startup checks: skein-llm-connection/src/component.rs:143-161, examples/echo/service/src/lib.rs:115-117, testing/skein-echo-client/src/lib.rs:226, tests/browser/tests/browser_real.rs:184, skein-oauth-accounts/src/limits.rs:39. Machines' own largest read: skein-http client.rs:101, server.rs:113, sse.rs:87, skein-json tokenizer.rs:81, skein-llm client.rs:1304.

## 4. Readers of unframed bytes (candidates; all already loop over the whole box)
1 smith-protocol-machine/src/process.rs:230,571 tool stdout/stderr; byte() :271-279 feeds rg search/listing parser or Capture (head+tail ring :64-118); head/tail <= output_bytes (:479); stream unbounded until deadline.
2 smith-host-protocol/src/process.rs:481 host stderr; keep_tail :486-501 builds a new List per delivery (once per byte today); detail_bytes (:117).
3 smith-local-protocol/src/git_child.rs:130,163 git stdout/stderr; stdout output_bytes+1 (:95); stderr detail_bytes.
4 smith-local-shell/src/front/headless/input.rs:58 stdin prompt until End; maximum.
5 smith-local-shell/src/local_auth_http.rs:330-341 HTTP body Fill(1) when chunked/until end; 16,384 (:296).
6 smith tests/local-process/src/terminal.rs:320-321 (called :333-334, :357) child stdout/stderr; 65,536 (:353).
7 skein-llm/src/client.rs:852,873 provider error body; error_bytes (:434).
8 skein-llm/src/client.rs:632 Sentinel [DONE] until End; 6.
9 skein-oauth-accounts/src/exchange.rs:340,370 token response body.
10 skein testing/skein-fake-peers/src/oauth.rs:250,339 request body.
11 skein testing/skein-fake-llm-protocol/src/provider.rs:609-616 chunked body.
12 jig-protocol-client-link/src/stream.rs:101-103 SSE event data -> base64 Decoder; document_bytes+512 (:118-121).
13 jig-shell-native/src/connections.rs:349-362 HTTP body Fill(1) when length unknown; body_bytes (:268).
14 jig-protocol-engine-clients/src/connection.rs:64-74 request body Fill(1) when chunked; http.body.
Test drivers: skein tests/world/src/bin/terminal_fixture.rs:47,173; tests/tls/tests/stack.rs:221; tests/oauth-accounts/src/{world.rs:1134,1138; sign_in/protocol.rs:291,295; sign_in/browser.rs:76,94}; jig tests/engine-clients/src/peer.rs:146; tests/channel/src/relay.rs:193.
NOT candidates (rely on exactness): smith terminal input smith-local-service/src/process.rs:254,272,829 (MAX_OUT 1 event per io event :788, terminal.rs:33-35; each LF pushes an event, Queue::push asserts room; Reading::Held back-pressure person.rs:166, process.rs:265-276 assumes one byte per delivery); skein-llm first SSE data byte client.rs:966 asserts len == 1; JSON tokenizer Fill(1) tokenizer.rs:587 (json.md 3.2 never past the document); HTTP body Fill(1) while ends_partway (body.rs:184-185); skein-channel frame header Fill(1) then Fill(7) (machine.rs:493-495).

## 5. Exhaustive matches on Read
Production 18: intake.rs:129-140, held.rs:40-64, stream.rs:999-1003, 1017-1037, 1038-1041, joined/end.rs:77-85, body.rs:174-199, 283-294, client.rs:844-848, server.rs:1032-1036, sse.rs:375-384, above.rs:167-170, close.rs:97-100, five tokenizer drivers (1b).
Tests/referees 21: skein-lib/src/tests/streams.rs:240-262, tests/lib/src/intake.rs:19-43, tests/io/src/ledger.rs:241-262, http client_world.rs:443-458, 1010-1014, 1063-1080; server_world.rs:613-627, 630-646, 1437-1441; sse_world.rs:550-560; json/src/world.rs:571-584, 600-606; tls/src/world.rs:237-251, 254-270, 680-687; tls/src/drive.rs:72-76; protocol/src/wire.rs:125-129; skein-tls/src/client/native/tests.rs:499-502; smith-local-shell/src/local_auth_http.rs:409-420 (cfg(test)).
Worlds' read-shape choices (Reads enums): client_world.rs:85,823-835; server_world.rs:108,1220; tls/src/world.rs:568; io/src/owner.rs:600-615.

## 6. Design text to amend (skein only)
lib.md 7: :142 enum; :151 "exactly the demand"; :165-166 answered "exactly what its read asks for"; :184-187 read larger than what is left; :216-217 Intake "exactly the demanded length"; :226-229 ends_partway "a byte at a time".
io.md 2 :141-147 startup check; 3 :152-156 sockets and pipes differ only in how made; 3.3 :324 Demand stored on ready list, :348-357 delivered step 1, :363-365 exactly, :385-386 scan max; 3.5 :477.
programming-model.md 4.3 :369-374 ("Bytes carries exactly the demanded bytes"); 8 :778-782.
testing-strategy.md 2.4 :111 ("a stream that delivers exactly what was demanded").
http.md 3.3 :230-241; 4 :409-412; 6 :895, :904, :936; 7 :1203-1211. json.md 3.2 :150-157, 3.1 :140, 6 :464. tls.md 3.1 :78-80, 3.2 :117-121. shell.md 8 :329. simulator.md 1 :20-22, 4 :168.

## 7. Simulator arrival sizes (skein-sim/src/)
Socket recv sim.rs:1269-1315 try_recv n = min(inbox, buf) :1296; short_recv seeded cut between(1, n-1) :1297-1299 (Fault::ShortRecv :1310-1311). Send cut() :1639-1645 at :1345, :1362 (short_send), capped by config.buffer :1358. Order :1168; latency :1443-1449. config.rs:43-46; NONE :94-112; CHAOS :115-128.
Pipes: no seeded cut: sim/processes.rs recv_inherited :363-376, recv_pipe :737-757, recv_bound :830-845 take min(buf, bytes); deliveries follow writer's writes (:759-805, :847-865).
World runs processes round-robin (skein-world/src/world.rs:396-409). Other seeded arrival: skein-world/src/stream.rs:87-88; test worlds' settings.piece (client_world.rs:628, server_world.rs:882, sse_world.rs:359, tls/src/world.rs:421, json/src/world.rs:399,718); jig test ends cut 1..17 (worker-link/src/joined.rs:49, net.rs:103, engine-workers/src/net.rs:105).

## Surprises
1 smith terminal input would panic on a delivery of two lines; back-pressure assumes one byte per delivery.
2 Stacked layers must pass the new read down as itself (HTTP body piece/answers); over TLS a delivery is at most one record's plaintext.
3 Tests/referees assert exactness and split-independence (streams.rs:336 every_split_of_the_input_meets_the_same_demands, :353; tests/lib/src/intake.rs:46-95; client_world.rs:655, server_world.rs:945, tls/src/world.rs:434; ledger.rs:240): need a "1..=max bytes, as they arrived" rule.
4 Pipes don't share stream::delivered (pipe.rs:328-341), pipe request path doesn't assert read bounds.
5 smith 6449a197 made the scripted person read by raw 4096-byte kernel PipeReads (terminal.rs:162-200); the new read could return it to io.
6 keep_tail allocates per delivery (smith-host-protocol process.rs:486-501).
