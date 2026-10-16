//! Engine-only native world configuration.
const ENGINE: &[u8] = br#"{
"limits": {"profile": "small", "overrides": {"tasks.result_bytes":32,"people.amendment_bytes":64,"core.policy_bytes":256,"tasks.spec_bytes":128,"tasks.message_bytes":512,"tasks.hold_bytes":64,"tasks.authority_bytes":64,"people.words":512,"core.call_answer_bytes":2048,"notes.recalled":1}},
"llm": {
"endpoints": [{
"name": "fake",
"number": 1,
"dialect": 0,
"account": 1,
"provider": "fake-provider",
"address": {"socket": "127.0.0.1:7101"},
"transport": "plaintext",
"headers": [],
"connect_ms": 10000,
"handshake_ms": 10000
}],
"models": [{
"name": "small",
"endpoint": "fake",
"window": 8192,
"output": 1024,
"prices": {"input": 1, "cached": 1, "output": 1, "unit": 1000}
}],
"accounts": [{
"number": 1,
"source": {"handed_in": {"file": "accounts/chat.json"}}
}],
"records": "accounts/records"
},
"hosts": {
"workers": [
{"name": "worker-a", "digest": "secrets/worker-a.digest"},
{"name": "worker-b", "digest": "secrets/worker-b.digest"}
],
"listener": [{"in_process": "workers"}]
},
"charters": [{
"number": 1, "instructions": "charters/chat.txt", "model": "small", "alternatives": [],
"hosts": "workers", "waiting_ms": 10000, "resume": true,
"turns": 4, "time_ms": 30000, "reserve_turns": 1, "reserve_time_ms": 5000, "spend": 20,
"inspect": true, "modify": false, "shell": false, "agents": false, "call_timeout_ms": 10000
}],
"templates": [
{"number": 1, "name": "chat", "executor": {"charter": 1}, "contract": {"report": {"words": 32}}, "shallower": 1, "budget": {"as_allowed": null}, "deadline": "none", "tracked": null, "conversation": true, "listed": false, "retention": "kept", "label": 1, "grace_ms": 5000},
{"number": 2, "name": "goal", "executor": "chosen", "contract": {"report": {"words": 32}}, "shallower": 1, "budget": "given", "deadline": "given", "tracked": {"priority": 1}, "conversation": false, "listed": true, "retention": "kept", "label": 2, "grace_ms": null}
],
"clients": {
"listener": {"socket": "127.0.0.1:7100"},
"heartbeat_ms": 10000,
"link": true
},
"deployment": {"directory": "state", "seed": 7},
"owners": [{"provider": 1, "subject": "owner-a"}],
"providers": [{"number": 1, "kind": "link"}, {"number": 2, "kind": "deployment"}],
"connectors": [1,2],
"memory_bytes": 1152921504606846976,
"rules": {
"ceiling": {"tools": 1023, "grants": [],
"delegation": {"kinds": [{"kind": "charter", "number": 1}], "tasks": 8, "depth": 8},
"budget": {"spend": 100, "deadline_unix_seconds": null}, "notes": 7, "note_resources": []},
"period_spend": 100, "minimum_run_spend": 1, "maximum_run_spend": 100,
"implies": [], "requirements": []
},
"projects": [{"number": 1, "roles": [{"connector": 1, "permission": 1, "role": 0}],
"policy": {"templates": [], "escalation_role": 0, "period_spend": 100, "projections": [], "requirements": [],
"ceiling": {"tools": 1023, "grants": [],
"delegation": {"kinds": [{"kind": "charter", "number": 1}], "tasks": 8, "depth": 8},
"budget": {"spend": 100, "deadline_unix_seconds": null}, "notes": 7, "note_resources": []},
"roles": [{"number": 0, "period_spend": 100, "requests": 511, "decides": 31,
"authority": {"tools": 1023, "grants": [],
"delegation": {"kinds": [{"kind": "charter", "number": 1}], "tasks": 8, "depth": 8},
"budget": {"spend": 100, "deadline_unix_seconds": null}, "notes": 7, "note_resources": []}}]
}}]
}"#;
pub(super) fn document() -> Vec<u8> {
    let mut bytes = b"{\"engine\":".to_vec();
    bytes.extend_from_slice(ENGINE);
    bytes.extend_from_slice(br#", "application":{"credential":"app.secret","connectors":[{"prefix":["one"],"resources":[]},{"prefix":["two"],"resources":[]}]},"service":{"components":["clients"]}}"#);
    bytes
}
