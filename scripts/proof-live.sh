#!/usr/bin/env bash
set -euo pipefail
umask 077
JIG_RUN_LOGS="$(realpath -m "${JIG_RUN_LOGS:-./logs/jig-26}")"
mkdir -p "$JIG_RUN_LOGS"
export JIG_LIVE_CONFIG="$JIG_RUN_LOGS/live-config.json"
JIG_LIVE_ACCOUNT="$(mktemp "$JIG_RUN_LOGS/live-account.XXXXXX")"
export JIG_LIVE_ACCOUNT
trap 'rm -f "$JIG_LIVE_ACCOUNT"' EXIT
python3 - <<'PY'
import base64, json, os, pathlib, socket, time, tomllib
try:
    login = json.loads((pathlib.Path.home() / '.codex/auth.json').read_bytes())
    token = login['tokens']['access_token']
    claim = token.split('.')[1]
    expiry = json.loads(base64.urlsafe_b64decode(claim + '=' * (-len(claim) % 4)))['exp']
    account = login['tokens']['account_id']
    if not isinstance(token, str) or not isinstance(account, str) or not account or expiry <= time.time() + 300:
        raise ValueError('borrowed login unavailable for five minutes')
    with open(os.environ['JIG_LIVE_ACCOUNT'], 'w') as file:
        json.dump({'access_token':token, 'expires_at':expiry, 'account_id':account}, file)
    os.chmod(os.environ['JIG_LIVE_ACCOUNT'], 0o600)
    model = os.environ.get('JIG_LIVE_MODEL')
    if not model:
        roots = [pathlib.Path.cwd(), *pathlib.Path.cwd().parents]
        models = next(root / 'smith/benchmarks/agents/models.toml' for root in roots
                      if (root / 'smith/benchmarks/agents/models.toml').is_file())
        model = tomllib.loads(models.read_text())['small']['codex']['model']
    address = socket.getaddrinfo('chatgpt.com', 443, socket.AF_INET, socket.SOCK_STREAM)[0][4][0] + ':443'
    with open(os.environ['JIG_LIVE_CONFIG'], 'w') as file:
        json.dump({'account_file':os.environ['JIG_LIVE_ACCOUNT'], 'model':model, 'address':address}, file)
    print('Live proof prepared: five completions maximum, ten minutes maximum; model=' + model)
except Exception:
    raise SystemExit('Cannot prepare the borrowed live account or public configuration') from None
PY
started=$SECONDS
status=0
timeout --signal=TERM --kill-after=10s 600s cargo nextest run -p jig-test --profile live --success-output immediate -E 'test(=a_native_client_s_chat_is_answered_by_a_real_model)' || status=$?
printf "Live proof script wall=%ss; exit=%s\n" "$((SECONDS-started))" "$status"
exit "$status"
