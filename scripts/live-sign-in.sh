#!/usr/bin/env bash
# The person-driven live layer (protocol/service.md, section 8): a fresh
# ChatGPT sign-in, a real model's answer and a confirmed sign-out.
set -euo pipefail
umask 077
cd "$(dirname "$0")/.."
printf '%s\n' 'The browser must reach 127.0.0.1:1455 on this machine: ssh -L 1455:127.0.0.1:1455 HOST'
export NEXTEST_DOUBLE_SPAWN=0
heavy cargo nextest run -p jig-test --profile live --no-run
# The test's scratch goes under this script's own directory, so that a run cut
# short (an interrupt, nextest's timeout) leaves no secret behind: the
# secrets' stores and the agent trace are erased however it ends.
scratch=$(mktemp -d)
erase() {
    find "$scratch" \( -type d -name secrets -prune -o -type f -name agent-trace.jsonl \) -exec rm -rf {} +
}
trap erase EXIT
status=0
TMPDIR=$scratch cargo nextest run -p jig-test --profile live --no-capture \
    -E 'test(=a_person_signs_in_to_chatgpt_and_a_real_model_answers)' || status=$?
erase
if ((status == 0)); then
    rm -rf "$scratch"
else
    printf '%s\n' "Diagnostics, without secrets: $scratch" \
        "A sign-in that did not sign out may still be listed at the issuer. Disconnect it in ChatGPT's settings, under Apps."
fi
exit "$status"
