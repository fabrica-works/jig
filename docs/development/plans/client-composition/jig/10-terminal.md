# Session 10: the terminal, and the worlds at the text shell

The text shell gets its terminal: lines read from an input pipe, pages
written to an output pipe, on skein's io, paced and one print at a time,
closing on `q`, the input's end or a termination signal. The client's
world plays two of its stories at the text shell, each act typed as the
line a person would type and read back by the command reader; the
client's protocol world runs one story with the terminal in front of its
native client, on skein's simulated machine; and `ops`'s client world
plays P's restart from the incident's page at the text shell, the text
shell's first consumer.

## Read first

- `client/domain/view.md`, 15.1 (the terminal, its settings), 15.2,
  15.5, 15.6 (its tests; "The client's worlds play the text shell").
- `client/domain/testing.md`, 2 (the protocol world's terminal), 4 ("At
  the text shell"), 9 (at the text shell, each story once, on one seed).
- `examples.md`, 8 ("Its terminal client").
- jig 09's interface sketch (`render`, `Print`, `command`, `Command`).
- `crates/jig-shell-native/src/files.rs:9,132-140` (how a component
  shares the kernel with io by tagging its operations' tokens).

## What the design asks

- `jig_shell_text::Terminal`: ordinary Rust, as shells are; it reads the
  input as lines, writes the output, and keeps when to print, on
  skein's io: its input adopted as a read pipe, its output as a write
  pipe. The application's host routes the terminal's io completions to
  it, gives it each tree the view builds and the lines the application
  prints of its own, and takes each command from it (15.1).
- `jig_shell_text::Settings`, given whole: the shell's name, the longest
  line it reads, its print interval (15.1).
- When it prints: the whole page; when the tree changed, at most once
  per print interval, a change within it printed when it ends; at once
  after the person's own command, once the steps that took it have built
  the tree, and at once on `p`; one print at a time; its own lines and
  the application's before the next print, each starting with the
  shell's name and `: ` (15.2).
- Its loop: a line is the bytes before `\n`, without a `\r` just before
  it; bytes with no `\n` wait; a line longer than the longest is refused
  whole, through its `\n`; each host iteration: the client's steps, the
  view built, each whole line made into one command and done, a print
  when due, the output written; closing on `q`, the input's end or a
  termination signal: no new print, what has begun is written, and it
  says it is closed (15.5).
- Tests: the terminal on skein's simulated machine (lines split and
  joined across reads, a line too long, prints paced and one at a time,
  the input's end closing); the client's worlds playing the text shell;
  the protocol world's one story with the terminal in front of its
  native client (15.6; testing.md, 2 and 4).

## Choices this plan makes

- **The terminal owns its own `skein_io::Io`,** and tags its operations'
  tokens with bit 62, as the native shell's files tag theirs with bit 63
  (`crates/jig-shell-native/src/files.rs:9`); the host gives it each
  completion for which `Terminal::owns(op)` holds. No change to
  `jig-shell-native` or to skein.
- **The host passes the tree to `iterate`**, which does 15.5's order
  after the host's steps: lines into commands (against that tree), a
  print when due, the output written. The commands wait in a queue the
  host drains.
- **At the text shell, the world** (`tests/client/src/harness/world.rs`)
  renders the client's tree and reads each act back: a person's act on
  node X with act A becomes the line for X's number (`n`, or `n WORDS`
  for an input), and `command` must give `Act { node: X, first: A, .. }`;
  a mismatch fails the run, naming the step. The world signs its clients
  in as it does today; the story starts on the page after signing in.

## Rules that bind it

- The commands are jig's: no application adds one (15.4).
- At the text shell the focused suite runs each story once, on one seed:
  in jig's client world, two stories (starting a chat; a held chat
  released); in `ops`'s, P's restart (testing.md, 9).
- Aim for the session's tests to add at most 0.8 s to the focused
  suite's median: a guide (README, "Test time"), checked by each
  landing's `Gate-Result`.

## Increments

### 10.1 The terminal
Scope: view.md, 15.1 (the terminal, its settings), 15.2, 15.5.
Needs: jig 09.1 and 09.2.
Files: `crates/jig-shell-text/src/terminal.rs`, `src/lib.rs`,
  `crates/jig-shell-text/Cargo.toml` (`skein-io`; dev: `skein-sim`,
  `skein-shell`, `skein-world`, as `crates/jig-shell-native/Cargo.toml`);
  `crates/jig-shell-text/src/tests/terminal.rs`;
  `tests/boundary/src/tests.rs` (the dependency list of 09.1's test
  gains `skein-io`).
Shape: as the sketch. `adopt` takes the input and output descriptors
  with `Io::adopt_read_pipe` and `Io::adopt_write_pipe(fd, None)`
  (skein `crates/skein-io/src/layer.rs:158,167` at jig's pin; no write
  timeout: a print waits on its reader, and the next waits for it,
  view.md 15.2). `iterate`
  reads what arrived, splits it into lines at `\n` (dropping a `\r`
  before it), makes each whole line a `Command` with `jig_shell_text::command`
  against the given tree and the last print's numbers, writes the
  shell's and the application's lines, then prints when one is due and
  no print is still being written. A print is due when the tree changed
  and the interval passed since the last print began; at once on `p`;
  and at the first `iterate` after a person's act command, since the
  host carries out the command after `iterate` returns and the next
  iteration's tree is the one its steps built (view.md 15.2: "once the
  steps that took it have built the tree"). `close`
  (the host heard a termination signal), `q` and the input's end close
  it: it starts no new print, writes what it began, and writes
  `NAME: closed`.
Test shape: `crates/jig-shell-text/src/tests/terminal.rs`, a simulated
  machine as `crates/jig-shell-native/src/tests.rs` builds one, its
  pipes from skein-sim's `Sim::open_inherited_read` and
  `open_inherited_write` (skein `crates/skein-sim/src/sim/processes.rs:268,273`):
  `a_line_split_across_reads_is_one_command`,
  `two_lines_in_one_read_are_two_commands`,
  `a_line_past_the_longest_is_refused_through_its_newline`,
  `changes_within_the_interval_print_once_when_it_ends`,
  `a_print_waits_for_the_last_to_be_written`,
  `the_applications_line_comes_before_the_next_print`,
  `the_inputs_end_closes_without_a_new_print`. Each under 0.5 s.
Proof: world, on skein's simulated machine: the tests above, once,
  `heavy cargo nextest run -p jig-shell-text`; 10.3 runs it in front of
  the real native client.
Don't: put the input in raw mode; sign anyone in or out.
Size: at most 250 lines of production code.
Removes: nothing.

### 10.2 The client's world at the text shell
Scope: testing.md, 4 ("At the text shell"), 9.
Needs: 10.1 is not needed; jig 09.2.
Files: `tests/client/src/harness/world.rs:327-338` (`act`, where a
  person's act reaches the view) and a setting that marks a client as
  played at the text shell (`Settings`, `:21-40`, or a `World` method
  beside `client_with`); `tests/client/Cargo.toml` (`jig-shell-text`);
  `tests/client/tests/stories.rs` (the two stories at the text shell).
Shape: `pub fn at_text_shell(&mut self, client: ClientId)`: from then on,
  each act the person makes on that client is checked as Choices says,
  then applied as today. An `Act::Input` on a field that submits on
  enter is held until the person's next act, which must be an
  `Act::Submit` on the same node: the pair is checked as the one line
  `n WORDS` (`Command::Act { first: Input, submit: true }`); any other
  next act fails the run with "typed without sending: not played at the
  text shell" (testing.md, 4).
Test shape: `tests/client/tests/stories.rs`:
  `starting_a_chat_at_the_text_shell` and
  `a_held_chat_is_released_at_the_text_shell`, each once on one seed,
  the steps of `signing_in_and_starting_a_chat` (`:7`, after its sign-in
  steps) and `a_held_chat_is_released` (`:53`, seed 11). Under 0.5 s
  each.
Proof: world, `heavy cargo nextest run -p jig-client-world -E
  'test(/at_the_text_shell/)'` once: each act of both stories is a line
  a terminal reads back as that act.
Don't: change the stories' steps; play any other story at the text
  shell in the focused suite.
Size: at most 60 lines of test kit code.
Removes: nothing.

### 10.3 The protocol world's terminal story
Scope: testing.md, 2 ("the text shell's terminal in front of the native
  one, on skein's simulated machine"); view.md, 15.6.
Needs: 10.1.
Files: `tests/client-protocol/src/native.rs` (a `NativeClient` that, when
  given a terminal, routes `Terminal::owns` completions to it, gives it
  each built tree, and turns its commands into acts on the view);
  `tests/client-protocol/src/native/world.rs:199-228` (`run`, which
  spawns the client with `world.spawn_root`; the client process's two
  inherited pipes, from `Sim::open_inherited_read`/`open_inherited_write`,
  as smith's `tests/local-process/tests/terminal.rs:13` opens one, read
  only as a model);
  `tests/client-protocol/src/stories.rs` or `native.rs`'s `Script` (the
  lines typed); `tests/client-protocol/tests/native.rs`;
  `tests/client-protocol/Cargo.toml` (`jig-shell-text`).
Shape: the story's person writes lines to the client's input pipe and
  reads pages from its output pipe; it starts a chat by its numbers, as
  `a_native_client_signs_in_by_the_service_s_link_and_starts_a_chat`
  (`tests/client-protocol/tests/native.rs:3`) does by the fake person.
Test shape: `tests/client-protocol/tests/native.rs`:
  `a_terminal_in_front_of_the_native_client_starts_a_chat` (the outcome
  counts one chat made, and the last page read from the output pipe
  holds the chat's number). Under 0.5 s.
Proof: world, the highest the protocol world reaches (the real native
  client and reference service on skein's simulated machine; a real
  process waits for `ops` standalone): this test, once, `heavy
  cargo nextest run -p jig-client-protocol-world -E 'test(terminal)'`:
  lines in on a pipe, pages out on another, through the real native
  client and the reference service.
Don't: add a fuzzy test.
Size: at most 120 lines of test code.
Removes: nothing.

### 10.4 `ops`'s terminal client in its world
Scope: examples.md, 8 ("Its terminal client … P accepts the restart
  from the incident's page, typing the numbers the page printed").
Needs: 10.2; jig 08.3 (the story on the incident's page).
Files: `tests/ops/tests/client_stories.rs`; `tests/ops/Cargo.toml`
  (`jig-shell-text`, if the world's use needs it).
Shape: no production code.
Test shape: `tests/ops/tests/client_stories.rs`:
  `p_accepts_the_restart_from_the_incidents_page_at_the_text_shell`:
  08.3's story, its client marked `at_text_shell`, seed 504, once.
  Under 0.5 s.
Proof: world, `heavy cargo nextest run -p jig-ops-world -E
  'test(at_the_text_shell)'` once.
Don't: give `ops` a command or a loop of its own (its loop over a real
  terminal waits for `ops` standalone, later.md).
Size: no production code.
Removes: nothing.

## Interface sketches

```rust
// crates/jig-shell-text/src/terminal.rs
/// What the application gives the terminal, whole, with no defaults.
pub struct Settings {
    /// The name its own lines start with.
    pub name: Box<[u8]>,
    /// The longest line it reads, in bytes.
    pub longest: u32,
    /// How often a changed tree is printed, at most (a guide).
    pub interval: Duration,
}
/// jig's terminal: lines in, pages out, on its own io, tagged.
pub struct Terminal { /* io, line buffer, last print, pacing, queues */ }
impl Terminal {
    pub fn new(settings: Settings, view: &jig_view::Limits) -> Option<Terminal>;
    /// Adopts the input and output pipes; a full io returns the descriptor.
    pub fn adopt(&mut self, input: Fd, output: Fd) -> Result<(), Fd>;
    /// Whether a completion's operation is the terminal's (bit 62 of its token).
    pub fn owns(op: Token) -> bool;
    pub fn completions(&mut self) -> &mut Queue<Complete>;
    pub fn submissions(&mut self) -> &mut Queue<Submit>;
    /// A line of the application's own, written before the next print.
    pub fn say(&mut self, line: &[u8]);
    /// One iteration, after the host's steps and the view's build.
    pub fn iterate(&mut self, now: Time, tree: &Tree, changed: bool);
    /// The commands read, in order, for the host to do.
    pub fn commands(&mut self) -> &mut Queue<Command>;
    /// The host heard a termination signal.
    pub fn close(&mut self);
    pub fn closed(&self) -> bool;
    pub fn work_pending(&self, now: Time) -> bool;
    pub fn next_deadline(&self) -> Option<Time>;
}
```

Who owns what: this session owns `Terminal` and `Settings`, and the
world's `at_text_shell`.

## Questions to raise when reached

- If skein-world's `World` (the protocol world's, `native/world.rs:199`)
  gives no access to the simulator for the client process's pid, so
  that its inherited pipes cannot be opened, 10.3 stops and asks: the
  change may be skein's, and is not this plan's to make.

- If the terminal cannot share the kernel by tagging its operations (an
  io operation's token above bit 62 in the native shell's io, say), the
  implementer stops and asks: the fix may belong in the native shell or
  upstream, and is not this plan's to choose.

## Done when

- `heavy cargo nextest run -p jig-shell-text -p jig-client-world -p
  jig-ops-world -p jig-client-protocol-world -E
  'package(jig-shell-text) | test(/text_shell|terminal/)'` passes on
  `main`, once.
