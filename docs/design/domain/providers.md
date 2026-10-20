# LLM providers' terms

Provisional, 2026-10-19. What OpenAI's and Anthropic's terms and
developer documentation allow an application on jig to do with a
person's subscription or API account, as read on 2026-10-17: who may
sign in and through which flow, which endpoints a token may reach, what
may be stored, lent or shared, and what a person must be shown. Sections
2 and 3 state the LLM providers' rules; section 4 lists what is still to
settle; section 5 gives the sources. How jig meets these rules, its
accounts and their sources, is accounts.md; how smith's agent runs
Claude Code as its backend is smith's `protocol/programs.md`. LLM
providers change these rules: read them again before a design relies on
them.

## 1. In one page

- **OpenAI sanctions plan usage in open-source and local tools,**
  through Sign in with ChatGPT: the tool registers itself at a person's
  first sign-in, runs its own OAuth exchange, and calls the public
  Responses API with the person's token (section 2). A paid or remotely
  hosted app asks OpenAI first.
- **Codex's own login is not for other clients.** A third-party client
  using Codex's Sign in with ChatGPT, and requests to ChatGPT's
  `backend-api` endpoints, are outside what OpenAI approves (2.1).
- **Anthropic allows no third-party subscription sign-in.** An
  application may not offer Claude.ai login, route requests through a
  Free, Pro or Max plan's credentials, or collect, store or intermediate
  Claude.ai credentials or session tokens. A product uses API keys
  (3.1).
- **Anthropic's one carve-out is Claude Code itself.** A person may sign
  in to the unmodified Claude Code binary with their own subscription,
  including where a product runs Claude Code, under conditions (3.2).
  Its login lives on each machine, kept by Claude Code (3.3).
- **API keys are the customer's to provision,** with either provider,
  billed to the key's owner (2.7, 3.1).
- **A subscription serves one person.** Neither provider lets a person
  share their account, and Anthropic's limits assume ordinary,
  individual use (2.6, 3.4).
- **Running a program as the agent's backend is smith's design**
  (smith's `protocol/programs.md`). For Claude, smith's agent runs
  Claude Code (`claude_code`) on a host where the person has signed it
  in with their subscription, in a configuration directory of the
  application's own, and calls Anthropic's API itself with an API key
  otherwise. The person signs in with Claude Code's own flow on each
  machine, and nothing of jig's or smith's ever reads that login
  (accounts.md, 3.4).

## 2. OpenAI

### 2.1 What is approved

- **Who may use a person's plan:** "ChatGPT plan usage is available to
  open-source projects, personal projects that run locally, and selected
  private apps." A paid or remotely hosted app completes OpenAI's
  interest form before offering it. Identity-only sign-in for commercial
  apps is a limited trial.
- **Codex's sign-in is Codex's.** Answering a third-party client that
  used Codex's Sign in with ChatGPT, OpenAI's Codex lead wrote on
  2026-09-08: "this is not an approved use of Sign in With ChatGPT. We
  support pure OSS clients and others we have partnerships with, but you
  need to reach out and talk to us for that." Plan usage for open-source
  tools, with a registration of their own, launched on 2026-09-29.
- **The public endpoint only:** inference goes to "the public Responses
  endpoint, not ChatGPT's backend-api endpoints".
- **What it grants:** eligible Responses API requests, counted toward
  the plan's included Codex usage. It grants no access to the person's
  conversations or other account context.
- **Eligibility:** Plus and Pro in the developer documentation; the help
  center also lists Go. A Free account may sign in but not use a plan.
  In a workspace, Sign in with ChatGPT is on unless its admins set an
  identity policy.

### 2.2 Registration and sign-in

- **First sign-in registers the tool,** with no client secret and no
  partner key:
  - `client_id=dynamic_agent_client`, the entry point only, never used
    for an exchange;
  - `agent_name_hint`: the tool's real name, used consistently, sent
    only for a new registration;
  - `ext_agent_host_id`: an opaque identifier per installation, chosen
    and kept before the first sign-in and sent again on every later one.
    `urn:ietf:params:oauth:jwk-thumbprint:…` is recommended,
    `urn:uuid:…` and `did:key:…` supported. Never an email, a user id or
    anything else that identifies the person; it is not a credential.
  - The callback returns the issued `client_id` (`oaiapp_…`) beside the
    code; the tool keeps it with the account's record and uses it for
    every exchange, refresh and revocation after. A callback without it
    is an incomplete registration.
- **Authorization** at `https://auth.openai.com/api/accounts/authorize`,
  opened in the system browser: `response_type=code`, a fresh `state`,
  `nonce` and PKCE verifier per attempt (`S256`), the scopes `openid
  profile email` for identity and `offline_access resource.invoke
  chatgpt.tokens.use.direct` for plan usage, and
  `resource=https://api.openai.com/v1`. Signing in again uses the issued
  client id, `id_token_hint` and `login_hint`, and no `agent_name_hint`.
- **The redirect** is an HTTP loopback callback on `127.0.0.1`, such as
  `http://127.0.0.1:1455/auth/callback`, bound before the browser opens.
  Only its port may change between sign-ins; its scheme, host and path
  may not, and `localhost` never stands in for `127.0.0.1`. The same URI
  goes in the request and the exchange.
- **The exchange** is a form-encoded `authorization_code` POST to
  `https://auth.openai.com/api/accounts/oauth/token` with the issued
  client id, the code, the verifier, the redirect URI and the resource.
  `invalid_grant` discards the code for a fresh authorization;
  `access_denied` at the callback, its state checked, stops without an
  exchange.
- **What the tool checks:** the ID token's signature against OpenAI's
  published keys, its issuer, audience (the issued client id), expiry
  and nonce; and that the granted scopes include
  `chatgpt.tokens.use.direct`, since a valid ID token alone authorizes
  no plan usage. The account is the validated `sub`, never the email;
  two registrations stay apart even when their emails match.
- **Declined plan usage** keeps the sign-in, marks plan usage off, and
  offers the person a choice: enable it, by signing in again with the
  full scopes, or another way to pay, such as their own API key.
- **No device-code flow** is documented for this program. A host with
  no browser gets its credential from a computer with one: OAuth there,
  with the same tool, user and workspace, then the credential file
  (client id, tokens, kept ID token) moved over a secure channel,
  keeping the host's own host id. Usage attribution and revocation per
  host are not available for a moved session.

### 2.3 Tokens

- **Lifetimes:** an access token lasts one hour; a refresh token 30
  days, and each refresh replaces it with a new one of 30 days, without
  limit while each stays valid. The token response carries
  `earliest_refresh_at`.
- **Refresh** with `grant_type=refresh_token`, the issued client id, the
  refresh token and the resource. "Serialize refreshes for the same
  session so two processes do not race a rotating token."
- **Storage:** a record per issued client id and account, protected,
  written atomically and `0600` on Unix; never in browser storage,
  source control, logs, URLs or support transcripts. The ID token is
  kept for `id_token_hint`; an authorization URL carrying one is
  redacted from logs.
- **Sign-out** revokes the refresh token at the revocation endpoint, an
  empty `200` meaning done; when revocation is not confirmed, the tool
  clears its tokens and tells the person to disconnect the app in
  ChatGPT's settings. OpenAI never tells the app that a person
  disconnected it: a refused request or refresh does.

### 2.4 Inference

- **The request:** `POST https://api.openai.com/v1/responses` with
  `Authorization: Bearer` and the access token; `store: false` and
  `stream: true` on every HTTP request; the whole context in `input`,
  since `previous_response_id` is not available over HTTP; `instructions`
  or developer messages, never system-role items.
- **Refused fields:** `background`, `conversation`, `max_output_tokens`,
  `max_tool_calls`, `metadata`, `moderation`, `multi_agent`, `prompt`,
  `prompt_cache_retention`, `safety_identifier`, `temperature`,
  `top_logprobs`, `top_p`, `truncation`, `user`.
- **Tools:** function and custom tools. Not image generation, file
  search, code interpreter, native computer use, hosted MCP or
  connectors, or `tool_search`; web search depends on the model and the
  account's policy. Inputs: text, images and files where the model takes
  them; no audio, video or file uploads.
- **Models:** `GET https://api.openai.com/v1/models` with the same
  token, keeping entries whose `visibility` is `list`, read again when
  the account changes. A listed model is a catalogue entry; only a
  completed request proves it usable.
- **Success** is `response.completed` alone: a `response.failed` may
  follow a stream that started, and `response.incomplete` or a stream
  that ends without completing is not success.

### 2.5 Errors

Plan usage errors stop the request: OpenAI never moves it to another way
of paying. The tool keeps the status, body, error code and request id.

| Error | Required behaviour |
|---|---|
| `subscription_sharing_usage_limit_exceeded` (429, or `response.failed`) | pause new plan requests; link to ChatGPT's usage settings; never claim the plan is empty or guess when it resets |
| `subscription_sharing_user_not_eligible` (403) | say why; never repeat the request or loop through sign-in |
| `subscription_sharing_usage_unavailable`, `subscription_sharing_user_unavailable` (503), and a bare 503 | keep the credentials; retry with bounded backoff |
| `subscription_sharing_unsupported_capability` (400) | remove what `error.param` names; never send the same body again |
| `subscription_sharing_route_not_supported` (403) | check the method and endpoint |
| `subscription_sharing_invalid_user` (401) | keep the request id and diagnose the credential; ask for a new sign-in only after a confirmed revocation or a terminal refresh error |
| `chatpass_v2_scope_not_authorized`, `chatpass_v2_invalid_authorization_context` (403) | check the client and the grant; never retry or change how it is paid |
| a refresh's `invalid_grant`, `invalid_refresh_token`, `token_expired`, `refresh_token_expired`, `refresh_token_invalidated`, `refresh_token_reused` | clear the unusable tokens; sign in again with the issued client id |
| a refresh's `invalid_client` | fix the client's configuration |

- **Never erase credentials** for a temporary network or service
  failure.

### 2.6 What the person sees, and sharing

- **The plan is required to be disclosed:** "Your app must clearly show
  which of its plans support ChatGPT plan usage", where it shows its own
  plans.
- **Recommended screens:**
  - "Continue with ChatGPT" on the action that starts the sign-in;
  - at the first sign-in with plan usage, once and dismissable, "You're
    using your ChatGPT plan", saying that eligible requests use the plan
    and that usage is managed in ChatGPT's settings;
  - "Using ChatGPT plan" near the composer or model picker while it is
    used, with a "Manage usage" link to `chatgpt.com/settings/usage`;
  - at a usage limit, "Manage usage" as the main action, a limit that
    may be the plan's or the app's alone;
  - plan usage kept distinct from the app's own charges.
- **Caps:** the person sets a weekly cap per app in ChatGPT's settings,
  as a share of their weekly usage; it reserves nothing. Plus has a
  five-hour limit shared by every app; Pro does not.
- **No sharing:** OpenAI's Terms of Use: "You may not share your account
  credentials or make your account available to anyone else and are
  responsible for all activities that occur under your account."

### 2.7 API keys

- **Platform API keys** pay at API prices, under the API organization's
  retention and data-sharing settings. Codex's own documentation calls
  them "the recommended default for automation".

## 3. Anthropic

### 3.1 Subscription credentials and API keys

From the authentication and credential use section of Claude Code's
legal and compliance page:

- **Subscription sign-in is for Anthropic's applications:** "OAuth
  authentication is intended exclusively for purchasers of Claude Free,
  Pro, Max, Team, and Enterprise subscription plans and is designed to
  support ordinary use of Claude Code and other native Anthropic
  applications."
- **Products use API keys:** "Developers building products or services
  that interact with Claude's capabilities, including those using the
  Agent SDK, should use API key authentication through Claude Console
  or a supported cloud provider."
- **No third-party sign-in:** "Anthropic does not permit third-party
  developers to offer Claude.ai login into their own applications, or to
  route requests through Free, Pro, or Max plan credentials on behalf of
  their users. Moreover, developers may not collect, store, or
  intermediate Claude.ai credentials or session tokens — sign-in to a
  Claude account must complete through Anthropic's own flow."
- **Keys are the customer's:** this does not restrict "configuring an
  API key in a development environment, secrets manager, or machine
  image for use by the customer's own authorized users", billed to the
  key's owner and not resold or intermediated.
- **Enforced without notice:** "Anthropic reserves the right to take
  measures to enforce these restrictions and may do so without prior
  notice." Reports date the enforcement against third-party tools from
  2026-01-09, and the page's wording from 2026-02-19.

### 3.2 Claude Code in a product

- **The carve-out:** the rules above do not "prevent an end user from
  signing in to the unmodified Claude Code binary with their own Claude
  subscription, including where a platform hosts Claude Code".
- **Running Claude Code in a product** ("preinstalling or running Claude
  Code in your products or services (e.g. in hosted sandboxes or other
  agent infrastructure)") requires agreeing to Anthropic's Commercial
  Terms of Service, and:
  - the binary runs as Anthropic publishes it, unmodified;
  - no built-in way of signing in is removed, disabled or restricted;
  - nobody pays for, resells or intermediates Claude usage on the end
    user's behalf;
  - each end user authenticates with their own API key, subscription
    plan credentials or cloud provider credential.
- **Names:** a product may say in plain text that it runs Claude Code,
  but may not use the Claude Code or Anthropic names or logos in its
  own product, feature or company name or logo, or suggest Anthropic's
  endorsement.

### 3.3 Claude Code's sign-in, per machine

- **Anthropic's flow:** `claude auth login` (or `/login` in a session)
  signs in; `claude auth logout` signs out. Where the browser cannot
  reach Claude Code's callback (SSH, containers, WSL2), it shows a code
  to paste into the terminal.
- **Status without credentials:** `claude auth status` prints JSON and
  exits 1 when not signed in; its `authMethod` is one of `none`,
  `claude.ai`, `oauth_token`, `api_key`, `api_key_helper` or
  `third_party`.
- **Kept by Claude Code, on each machine:** the macOS Keychain, or
  `~/.claude/.credentials.json` (`0600`) on Linux, and the same file
  under `%USERPROFILE%` on Windows. `CLAUDE_CONFIG_DIR` names a
  configuration directory with its own settings, sessions and login. A
  login expires: Claude Code warns three days before, and then fails
  each request with "Login expired · Please run /login".
- **A long-lived token** from `claude setup-token`, for CI and scripts:
  printed, never saved, set by the person as `CLAUDE_CODE_OAUTH_TOKEN`.
  An application that stored it or passed it on would hold a Claude.ai
  session token (3.1).
- **Which credential wins:** a cloud provider's, then
  `ANTHROPIC_AUTH_TOKEN`, `ANTHROPIC_API_KEY`, an `apiKeyHelper`,
  `CLAUDE_CODE_OAUTH_TOKEN`, Anthropic profiles, and last the
  subscription login. In `-p`, `ANTHROPIC_API_KEY` "is always used when
  present".

### 3.4 Usage and billing

- **Ordinary, individual use:** "Advertised usage limits for Pro and Max
  plans assume ordinary, individual usage of Claude Code and the Agent
  SDK."
- **Programmatic use draws from the plan:** Anthropic's help center says
  "Claude Agent SDK, `claude -p`, and third-party app usage still draw
  from your subscription limits"; since 2026-10-07, Max and Team plans
  also include monthly API credits covering the Agent SDK, `claude -p`,
  the Claude API and Managed Agents. A separate credit for programmatic
  use, announced for 2026-06-15, is paused.

### 3.5 Scripted runs

- **`--bare` reads no subscription login:** "In bare mode, Claude Code
  never reads OAuth credentials or the system keychain". It is "the
  recommended mode for scripted and SDK calls, and will become the
  default for `-p` in a future release".
- **Without `--bare`, a repository's settings run:** "a `-p` session
  runs the hooks in a project's `.claude/settings.json` and connects the
  servers in its `.mcp.json`, even in a folder you've never trusted."
- **What a script controls:** `--output-format stream-json`,
  `--input-format stream-json`, `--include-partial-messages`,
  `--json-schema`, `--permission-mode`, `--permission-prompt-tool`,
  `--permission-prompts none`, `--tools`, `--allowedTools`,
  `--disallowedTools`, `--mcp-config` with `--strict-mcp-config`,
  `--setting-sources`, `--max-turns`, `--max-budget-usd` (a client-side
  estimate), `--session-id` and `--resume`, which also takes a
  transcript file's path. SIGTERM exits 143 with the turn unfinished;
  SIGINT ends the turn.

## 4. Open questions

- **"Open-source project":** what OpenAI counts as one ("pure OSS
  clients"). skein, smith and jig take the MIT licence.
- **Unattended agents on a plan:** OpenAI's plan usage documents do not
  address background or unattended use; Anthropic's limits assume
  ordinary, individual use (3.4).
- **A product that runs Claude Code for its person:** whether an
  application spawning the person's own, unmodified, signed-in
  `claude -p` is the carve-out of 3.2, and whether its developers then
  need Anthropic's Commercial Terms, given that developers building on
  the Agent SDK "should use API key authentication" (3.1). Anthropic's
  legal page offers a contact for questions about permitted
  authentication methods.
- **Go plans:** the developer documentation names Plus and Pro, the
  help center Go, Plus and Pro (2.1).

## 5. Sources

Read on 2026-10-17. OpenAI's help center and Terms of Use refused a
direct read; what this document takes from them comes through search
results and the trackers below.

- OpenAI: [Sign in with ChatGPT](https://developers.openai.com/siwc);
  [plan usage overview](https://developers.openai.com/siwc/token-sharing-open-source);
  [registration and sign-in](https://developers.openai.com/siwc/token-sharing-open-source/sign-in);
  [accounts and sessions](https://developers.openai.com/siwc/token-sharing-open-source/profiles-and-sessions);
  [token reference](https://developers.openai.com/siwc/token-sharing-open-source/token-reference);
  [models and inference](https://developers.openai.com/siwc/token-sharing-open-source/models-and-inference);
  [errors and recovery](https://developers.openai.com/siwc/token-sharing-open-source/errors-and-recovery);
  [preview limitations](https://developers.openai.com/siwc/token-sharing-open-source/preview-limitations);
  [self-hosted VMs](https://developers.openai.com/siwc/token-sharing-open-source/self-hosted-vms);
  [Codex app-server](https://developers.openai.com/siwc/token-sharing-open-source/codex-app-server);
  [UI/UX guidelines](https://developers.openai.com/siwc/ui-ux-guidelines);
  [cookbook: Sign in with ChatGPT in an open-source app](https://developers.openai.com/cookbook/articles/sign-in-with-chatgpt);
  [Codex authentication](https://learn.chatgpt.com/docs/auth);
  [using your ChatGPT plan in other apps](https://help.openai.com/en/articles/20001542-using-your-chatgpt-plan-in-other-apps-and-sites);
  [Terms of Use](https://openai.com/policies/terms-of-use/), its
  account-sharing clause as quoted by
  [Conduct Atlas](https://conductatlas.com/platform/openai/terms-of-use-row/provision/CA-P-017119/account-credential-sharing-prohibited/).
- OpenAI's Codex lead on
  [X, 2026-09-08](https://x.com/thsottiaux/status/2097131394199896166).
- Anthropic: [Claude Code legal and compliance](https://code.claude.com/docs/en/legal-and-compliance);
  [authentication](https://code.claude.com/docs/en/authentication);
  [CLI reference](https://code.claude.com/docs/en/cli-reference);
  [running Claude Code programmatically](https://code.claude.com/docs/en/headless);
  [the Agent SDK with a Claude plan](https://support.claude.com/en/articles/15036540-use-the-claude-agent-sdk-with-your-claude-plan).
- Reports: [The Register, 2026-02-20](https://www.theregister.com/software/2026/02/20/anthropic-clarifies-ban-on-third-party-tool-access-to-claude/5014546);
  [Zed, on Anthropic's billing change](https://zed.dev/blog/anthropic-subscription-changes).

