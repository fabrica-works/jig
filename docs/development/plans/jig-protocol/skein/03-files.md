# Session 03: opening a file to write, and cutting it

The kernel boundary gains the `Write` open, which opens an existing
regular file to read and write and never makes one, with its form that
refuses a symbolic link, and `Truncate`, which sets an open file's
length; both backends hold them, the simulator's crashing disk keeping a
file's old length until the file is synced. io's file driver then offers
them as `OpenWrite`, `OpenWriteNoFollow` and `Cut`. jig's store needs
them to resume a transcript's file after a restart: open it, cut it to
its committed length, sync it, write on (jig's `protocol/store.md`,
section 8; skein's `io.md`, 5). Two increments: the kernel and its
backends, then io.

## Read first

- skein's `kernel.md`, 6.1 (the opens io makes, "Cutting a file", which
  operations may be cancelled, errors named per operation) and section 8
  (the conformance suite, files beneath a scenario's own root).
- skein's `simulator.md`, 3.1 (the machine seam, for files) and 3.3 ("a
  file's length cut (kernel.md, 6.1) only once the file was synced, the
  old length kept until then").
- skein's `io.md`, section 5 (the sketch's `OpenWrite`, `Cut`; "Writing a
  file that exists"; one request at a time, each with a deadline).
- `findings/skein.md`, section 6 (the code today, the exhaustive matches
  that break, the risks).

## What the design asks

- **The `Write` open** (kernel.md, 6.1): `OpenHow` gains "`Write` (an
  existing regular file, to read and write, never created), each with a
  form that refuses a symbolic link, as `Read`'s does".
- **`Truncate`** (kernel.md, 6.1): "on a descriptor opened to write
  (`ftruncate`): the file's length becomes the one given, and what lay
  past it is gone. Like a write, it is durable once the file is synced;
  the simulator's crashing disk keeps the old length past a crash until
  then".
- **The crash** (simulator.md, 3.3): after a power loss the machine
  keeps "a file's length cut ... only once the file was synced, the old
  length kept until then. The choices are drawn from the seed."
- **io** (io.md, 5): "`OpenWrite` opens an existing regular file to read
  and write, never creating it (`Missing` if it is absent), so a writer
  that resumes a file after a restart writes at the offsets it chooses;
  `Cut` sets an open file's length, dropping what lies past it, durable
  once the file is synced." Each request ends in one event: its own,
  `Failed { error }` or `Cancelled`.

## Choices this plan makes

- `kernel::OpenHow::{Write, WriteNoFollow}`; `kernel::Op::Truncate { fd:
  Fd, length: u64 }`, answered `Done::Nothing`; a `length` past
  `i64::MAX` is an invalid record, as an offset is (kernel.md, 6.1).
- On the ring, `Write` is `O_RDWR` with no `O_CREAT`, through the same
  `openat2` and `NotAFile` check as `Read` (`ring.rs:1282-1299`), and
  `WriteNoFollow` adds `RESOLVE_NO_SYMLINKS` as `ReadNoFollow` does;
  `Truncate` is io_uring's `IORING_OP_FTRUNCATE` (io-uring 0.7.15's
  `opcode::Ftruncate`; Linux 6.9, below the 6.12 floor of shell.md, 4),
  added to the probe list (`ring.rs:383-401`).
- A descriptor opened with `Write` takes `Read`, `Write`, `Truncate`,
  `Sync`, `Stat` and `Close`; `Truncate` on one opened otherwise is a
  broken invariant (the list in `kernel.rs`'s module documentation,
  `:217-251`, gains it), which the simulator fails the world on.
- `Truncate`'s errors: `ReadOnly` (`EROFS`), `Permission` (`EPERM`, a
  file the system holds append-only or immutable) and `NoSpace`
  (`ENOSPC`); `EFBIG` is `InvalidArgument`, as `EINVAL` is.
- `Truncate` is not cancellable: kernel.md, 6.1 names "An `Open`,
  `Read`, `Write`, `Append` or `Sync`" as those that may be, and not it.
  A `Cancel` of one is a broken invariant; the simulator draws no `hung`
  fault for it; io waits for it at a `Cut`'s deadline, as it waits for a
  `Stat`, then answers `Failed { error: TimedOut }`. See Questions.
- io's "`Missing`" is `Event::Failed { error: NotFound }` (io.md, 5's
  rule: "each request ends in one event: its own, `Failed { error }` or
  `Cancelled`").
- io's `OpenWrite` answers `Opened { owner, file, len }` as `OpenRead`
  does, so a resuming writer learns the file's length; one function opens
  for both (`open_read`, `file_layer.rs:611-631`, takes the `OpenHow`),
  not a copy.
- The fake machine records a pending cut per node, as it records pending
  writes (`fs.rs:440-460`): `sync` makes it durable; `crash` drops it,
  so the durable node's old length and bytes stand, and the pending
  writes drawn land on them.

## Rules that bind it

- "**Shared hotspots with smith's reliability plan,** if it runs at the
  same time on another machine: skein's `crates/skein-io/src/kernel.rs`,
  `crates/skein-io/src/file*.rs` and `crates/skein-shell/` ... `land`
  rebases each landing on the other's; no session waits for it" (README,
  "Rules"). That plan's skein session adds a kernel `Window` record to
  the same matches 03.1 extends: resolve its arms beside `Truncate`'s.
  Within this plan, skein 02.3 edits `crates/skein-shell/src/ring.rs`
  too.
- "**No increment of this plan** ... leaves a legacy or compatibility
  path past the increment that its session file names to remove it"
  (code-organization.md, section 6): each match that gains `Truncate`
  handles it, none with a catch-all.
- Files already past 800 lines grow by at most the increment's Size and
  are not split here, replacing code-organization.md, section 7's split
  for them: `crates/skein-io/src/kernel.rs` (1,484),
  `crates/skein-io/src/file_layer.rs` (1,273),
  `crates/skein-shell/src/ring.rs` (1,731), `crates/skein-sim/src/sim.rs`
  (1,746), `testing/skein-fake-machine/src/fs.rs` (1,057),
  `testing/skein-conformance/src/run.rs` (949); the conformance scenario
  goes in a new module.

## Increments

### 03.1 The kernel's `Write` open and `Truncate`, on both backends
Scope: kernel.md, 6.1's `Write` and `WriteNoFollow` opens and
  `Truncate`; simulator.md, 3.1 and 3.3; kernel.md, 8's scenario on both
  backends.
Needs: nothing.
Files:
  - skein-io: `src/kernel.rs` (`OpenHow` at `:591-610`; `Op` at
    `:364-470`; `is_valid` `:931,953`; `shape` `:995,1012`; `files`
    `:1023-1028`; `fits` `:1331-1336`; `binds` `:1377-1382`; `counts`
    `:1402-1423`; the module documentation's error table at `:205-213`
    and broken invariants; `Error`'s docs); the matches of `Op` in
    `src/file_layer.rs:827-832`, `src/file_layer/store.rs:671-676,703-708`,
    `src/pipe.rs:523-540`, `src/stream.rs:682-687,711-716`,
    `src/tests.rs:71-76,131-139`, `src/tests/kernel.rs:81-98,297-319,368-378`,
    `src/tests/output.rs:329-334`, `src/tests/store.rs:380-382`.
  - skein-shell: `src/ring.rs` (`prepare` `:807-829`, `decode`
    `:918-957`, `error` `:980-985`, `operation_error` `:1071-1076`,
    `file_error` `:1113-1146`, `check_path_access` `:1211-1219`, `open`
    `:1282-1299`, the probe list `:383-401`).
  - skein-sim: `src/machine.rs:57-95` (`Ask::Truncate`); `src/sim.rs`
    (`cut_with` `:275-280`, `submit_one` `:571-576`, `withdraw`
    `:1161-1166`, access `:717-773`); `src/sim/files.rs:117-121,153-160,172-187,246-275,344-358`;
    `src/trace.rs:219-228` and `Summary::Truncate`.
  - `testing/skein-fake-machine/src/fs.rs` (`How` `:60-80`; `truncate`
    beside `write` `:440-460`; `sync` `:469-485`; `crash` `:487-534`),
    `src/face.rs:35-60,139-147`, `src/tests.rs`.
  - `testing/skein-conformance/src/run.rs` (`:116-121,751-760,792-807,842-847`,
    and a `truncate` helper beside `write_all` at `:563`),
    `src/files.rs:23` (`rule` made `pub(crate)`) and `:773-777`, new
    `src/cuts.rs`, `src/lib.rs` (exports);
    `tests/conformance/ring/tests/conformance.rs`,
    `tests/conformance/sim/tests/conformance.rs`.
  - Other matches of `Op`: `examples/echo/service/src/tests.rs:149-154`,
    `testing/skein-echo-client/src/tests.rs:167-172`,
    `testing/skein-world/src/real.rs:609-614`,
    `tests/io/tests/memory.rs:221-265`, `tests/ring/src/lib.rs:172-177`;
    `tests/sim/tests/{files,invariants}.rs`.
  - Lines are findings/skein.md's, at skein `5c9eedd`, code unchanged at
    `917ea21`; `heavy cargo check --workspace --all-targets` lists any
    match they miss.
Shape:
  - `OpenHow::Write`, documented "An existing regular file, to read and
    write (`O_RDWR`), never created"; `OpenHow::WriteNoFollow`, "An
    existing file to read and write, refusing a symbolic link in any path
    part".
  - `Op::Truncate { fd: Fd, length: u64 }`, documented "Sets the length
    of the file open on `fd`, opened with `Write`: what lay past it is
    gone; durable once synced"; `Op::is_file` holds for it; it names `fd`
    for the descriptor rules.
  - `skein_sim::Ask::Truncate { file: Handle, length: u64 }`, answered
    `Reply::Done`.
  - `skein_fake_machine::How::{Write, WriteNoFollow}`;
    `Machine::truncate(&mut self, file: Opened, length: u64) ->
    Result<(), Refusal>`, the length past `LARGEST_FILE` refused
    `NoSpace` as a write's is (`fs.rs:448-450`).
Test shape:
  - Conformance, new `testing/skein-conformance/src/cuts.rs`, run on the
    ring and in the simulator. Reference scenario and its two runs:

    ```rust
    /// What `Write` opens found and what `Truncate`s left (kernel.md, 6.1).
    #[derive(Debug)]
    pub struct Cutting {
        pub missing: Result<Fd, Error>,
        pub after_missing: Result<Vec<u8>, Error>,
        pub shortened: Vec<u8>,
        pub lengthened: Vec<u8>,
        pub refused: [Error; 3],
    }

    /// A `Write` open of a missing file makes none; one of a file opens it;
    /// a `Truncate` shortens it, then lengthens it with zeros.
    #[must_use]
    pub fn cutting<B: Backend>(backend: &mut B) -> Cutting {
        let mut run = Run::new(backend);
        let process = run.process();
        let tree =
            [Item::file(b"f", b"0123456789"), Item::directory(b"d"), Item::fifo(b"pipe"), Item::link(b"inside", b"f")];
        let root = run.root(process, &tree);
        let missing = run.open(process, root, b"missing", OpenHow::Write);
        let after_missing = run.contents(process, root, b"missing");
        let fd = run.open(process, root, b"f", OpenHow::Write).expect("an existing file opens to write");
        run.truncate(process, fd, 4).expect("cut to four bytes");
        let shortened = run.contents(process, root, b"f").expect("read back");
        run.truncate(process, fd, 6).expect("cut past the end");
        let lengthened = run.contents(process, root, b"f").expect("read back");
        run.close(process, fd);
        let refused = [
            run.open(process, root, b"d", OpenHow::Write).expect_err("a directory does not open to write"),
            run.open(process, root, b"pipe", OpenHow::Write).expect_err("a FIFO opens as no file"),
            run.open(process, root, b"inside", OpenHow::WriteNoFollow).expect_err("no link is followed"),
        ];
        run.close(process, root);
        run.finish();
        Cutting { missing, after_missing, shortened, lengthened, refused }
    }

    impl Check for Cutting {
        fn check(&self) {
            rule(self.missing == Err(Error::NotFound), "Write never creates: a missing file is NotFound", self);
            rule(self.after_missing == Err(Error::NotFound), "and nothing was made", self);
            rule(self.shortened == b"0123", "Truncate drops what lay past the length", self);
            rule(self.lengthened == b"0123\0\0", "Truncate past the end reads as zeros", self);
            rule(
                self.refused == [Error::IsADirectory, Error::NotAFile, Error::TooManyLinks],
                "Write refuses a directory and a FIFO, and its no-follow form a link",
                self,
            );
        }
    }

    // tests/conformance/ring/tests/conformance.rs
    #[test]
    fn a_write_open_finds_what_exists_and_truncate_sets_its_length() {
        on_the_ring(skein_conformance::cutting);
    }

    // tests/conformance/sim/tests/conformance.rs
    #[test]
    fn a_write_open_finds_what_exists_and_truncate_sets_its_length() {
        calm_and_chaos(skein_conformance::cutting);
    }
    ```

  - Fake machine, `testing/skein-fake-machine/src/tests.rs`, after
    `recovered_hard_link_counts_follow_the_synced_namespace` (`:293-307`):
    `a_cut_not_synced_keeps_the_old_length_past_a_crash` (over seeds 0 to
    15, the file reads `0123456789` after the crash) and
    `a_cut_synced_keeps_the_new_length_past_a_crash` (`0123`).
  - Simulator, `tests/sim/tests/invariants.rs`:
    `a_truncate_on_a_descriptor_opened_to_read_fails_the_world`,
    `a_cancel_of_a_truncate_fails_the_world`.
  - skein-io, `src/tests/kernel.rs`'s tables (`:297-319`): Truncate's row
    of errors, and `a_truncate_past_i64_max_is_an_invalid_record`.
  - Each under 20 ms; the conformance runs under 0.5 s each.
Proof: world (simulated io) and the real kernel, run once with `heavy
  cargo nextest run -p skein-conformance-sim -p skein-conformance-ring
  -p skein-fake-machine`: the scenario's rules hold on the ring and over
  the simulator's calm and chaotic seeds, and a cut the file was not
  synced after is undone by a crash. Under 1 minute.
Don't: let `Create` or `Read` take `Truncate`; draw `hung` for a
  `Truncate`; add an io request (03.2).
Size: at most 280 lines of production code (the kernel's records and
  docs, the ring, the simulator, the fake machine; the conformance
  scenario is test code).
Removes: nothing.

### 03.2 io's `OpenWrite`, `OpenWriteNoFollow` and `Cut`
Scope: io.md, 5: "Writing a file that exists", the sketch's
  `OpenWrite`, `OpenWriteNoFollow` and `Cut`, one request at a time with
  a deadline.
Needs: nothing beyond 03.1.
Files: `crates/skein-io/src/file.rs:31-148` (three requests),
  `:175-260` (`Event::Cut`), `:262-275` (`owner()`);
  `src/file_layer.rs:440-565` (`down_inner`), `:611-631` (`open_read`
  takes the `OpenHow`), `Pending` (a `Cut { owner }`; `can_cancel` false
  for it), `up_inner` (`Cut` told); `src/tests/file.rs`. Users of the
  exhaustive matches: `crates/skein-oauth-accounts/src/private_tests.rs:105-115,331-339`
  and `testing/skein-world/src/files.rs:214-224` (`file::Request`);
  `crates/skein-oauth-accounts/src/component/private.rs:195-200,236-241,286-290,377-381`
  (`file::Event`). `kv`'s tuple matches have catch-alls: no edit.
Shape:
  - `Request::OpenWrite { owner: Token, root: Fd, name: Box<[u8]> }` and
    `Request::OpenWriteNoFollow { owner: Token, root: Fd, name: Box<[u8]>
    }`, as `OpenRead`'s (roots by descriptor, io.md, 9); answered
    `Opened { owner, file, len }` or `Failed`.
  - `Request::Cut { owner: Token, file: Token, length: u64 }`, answered
    `Event::Cut { owner: Token }` or `Failed`; a file token io does not
    hold is `Failed { error: NotFound }`, as `WriteAt`'s is.
Test shape: `crates/skein-io/src/tests/file.rs`, its `io()` (`:17-19`),
  `complete` (`:34-42`), `OWNER` and `ROOT` (`:13-14`). Reference test:

  ```rust
  #[test]
  fn open_write_asks_the_write_open_and_tells_not_found_as_failed() {
      let mut io = io();
      let mut events = Queue::with_capacity(2);
      let mut subs = Queue::with_capacity(2);
      let request = Request::OpenWrite { owner: OWNER, root: ROOT, name: Box::from(&b"log"[..]) };
      file_layer::down(&mut io, Time::ZERO, request, &mut events, &mut subs);
      let submit = subs.pop().expect("one open");
      let Op::Open { how, .. } = &submit.kind else { panic!("an open") };
      assert_eq!(*how, OpenHow::Write, "an existing file, to read and write, never made");
      complete(&mut io, submit, Err(crate::kernel::Error::NotFound), &mut events, &mut subs);
      assert_eq!(
          events.pop(),
          Some(Event::Failed { owner: OWNER, error: crate::kernel::Error::NotFound, committed: false, residue: None })
      );
      assert!(io.takes(), "the driver takes the next request");
  }
  ```

  The others: `open_write_tells_the_opened_files_length`,
  `open_write_no_follow_asks_the_no_follow_write_open`,
  `a_cut_submits_a_truncate_and_tells_cut`,
  `a_cut_of_a_file_io_does_not_hold_fails_not_found`,
  `a_cut_past_its_deadline_waits_for_its_truncate_then_times_out`; each
  under 20 ms.
Proof: unit, run once with `heavy cargo nextest run -p skein-io --lib
  file`: a writer's `OpenWrite` and `Cut` reach the kernel as the `Write`
  open and a `Truncate` and are told back as `Opened` with the length,
  `Cut`, or `Failed`. Under 30 s. Not a world: io's worlds hold no file
  scenarios yet (io.md, 9); kv's and jig's store's worlds exercise them
  through the simulator later.
Don't: make `Cut` cancellable; add `Cut` for a root's descriptor.
Size: at most 90 lines of production code.
Removes: nothing.

### 03.s1 A directory io makes, synced into its parent; `make_root`

A step not in the plan (decisions.md, rulings 241 and 245), from the
designer's answer on a directory io makes (skein `63e81ded`: `io.md`, 5,
"What a request makes by a path, io makes durable", 5.3 and 8;
`shell.md`, 6, `make_root`). jig's store lost `store/`, `transcripts/`
and `secrets/` to a power loss after an acknowledged commit (jig 18.s1,
18.s2); skein-oauth-accounts' private root, which smith's credentials
open through it, has the same gap; and skein's crash worlds lay every
directory before they start, so none was cut.
Needs: nothing.
Files: `crates/skein-io/src/file_layer/private.rs` (`up`, :201-283) and
  its step tests; `crates/skein-shell/src/ring.rs` (`make_root`, beside
  `open_root`, :1593); `tests/io/src/cuts.rs`;
  `tests/oauth-accounts/src/private.rs`.
Shape: in `private::up`, once a `MakeDirectory` io issued completes
  (`Phase::Making`, for `Action::Make` and `Action::Private` alike), io
  issues `Op::Sync` of the parent it holds (a phase of its own), and
  only then answers `Made`, or opens the private root; a failed sync
  answers `Failed { committed: true }`, the parent closed as now. A
  private root found (opened at once, or `Exists` while making) is not
  synced. `skein_shell::make_root(path: &Path, mode: u32) -> Result<Fd,
  i32>`: from the first existing ancestor, makes each absent component
  with `mode`, syncs each directory it made one in, then opens the last
  as `open_root` does; an existing component is never changed.
Test shape: step tests
  `a_made_directory_answers_after_its_parent_is_synced`,
  `a_private_root_made_is_synced_before_it_is_opened`,
  `a_private_root_found_is_not_synced`,
  `a_failed_parent_sync_answers_committed`; in io's cut world and the
  oauth accounts' private world, a story whose private root starts
  absent, cut at every step and at a power loss: after the cut the root
  is there whenever its open was answered, and the record whenever its
  store was; `make_root` on the real kernel: a nested absent path made
  and opened, a file in its way refused.
Proof: unit, `heavy cargo nextest run -p skein-io -p skein-io-world -p
  skein-oauth-accounts-world` once: the new story, run before the
  change, loses the root at the power loss after the make, and passes
  after it; each test under 20 ms.
Don't: sync a directory io did not make; change an existing directory's
  mode; answer `Made` for one that existed.
Size: at most 120 lines of production code.

### 03.s2 The simulator settles a child its owner ended

A step not in the plan, from the designer's answer on jig 26.2's worker
abort (skein `b638df3c`: `testing-strategy.md`, 6; `io.md`, 3, "Abort
closes at once", and 6, closing a child kills its group). jig 26.2
aborts a worker's children and their pipes, as `service.md`, 10 says,
and the simulator then fails the world: `Sim::assert_hosted_settled`
(`crates/skein-sim/src/sim/processes.rs:248-260`) asserts every hosted
child's output pipe read to its end, whatever ended the child.
Needs: nothing.
Files: `crates/skein-sim/src/sim/processes.rs`;
  `tests/sim/tests/processes.rs` (`skein-sim-tests`).
Shape: the read-to-end assertion holds for a child whose exit is
  `Exit::Code`; a child whose exit is `Exit::Signal` (its owner closed
  it, killing its group, or signalled it) may leave its output pipes
  unread. Every child still has exited.
Test shape: `a_killed_childs_unread_output_is_settled` (a hosted child
  writes past what its parent reads; the parent aborts the pipe and
  closes the child; the world settles); and
  `a_childs_own_exit_with_unread_output_is_caught` (a child exits with a
  code, its output unread: the assertion's message). Under 20 ms each.
Proof: unit, `heavy cargo nextest run -p skein-sim-tests --test
  processes` once.
Don't: count a closed pipe as read; loosen the check for a child that
  exited with a code.
Size: at most 20 lines of production code.

### 03.s3 A power loss restarts the world's own processes, not hosted children

A step not in the plan, from the designer's answer on jig 26.3's power
loss (skein `2f89cf78`: `simulator.md`, 3.3). On `Cut::PowerLoss`,
`World::cut` and `perform_cut`
(`testing/skein-world/src/world.rs:279-287`, `:630-679`) ask a restart factory of every admitted process and restart
all of them; `World::host` registers a hosted child's launch factory
and no restart slot (`:239-255`; `admit` pushes `None`, `:338`), so no
world with a live hosted child can be cut by power loss.
Needs: nothing.
Files: `testing/skein-world/src/world.rs`; `tests/world/tests/hosted.rs`
  (`skein-world-tests`).
Shape: on `Cut::PowerLoss`, every process is dropped at the cut as now
  (`sim.cut_with`, its held handles given to the machine's cut); each
  with a restart factory restarts as now; each hosted child (its
  `hosted` slot `Some`) leaves the world's tables, as a killed one does
  (`drop_killed`, `:585`), and is not started again. `cut` and
  `perform_cut` ask a factory of every process but hosted children. A
  kill is unchanged.
Test shape: `a_power_loss_drops_hosted_children_and_restarts_the_roots`
  (a restartable root spawns a hosted child; power is cut while the
  child runs; the root restarts, the child does not, the root spawns it
  again, and the world settles);
  `a_power_loss_needs_every_roots_factory` (a process the world started
  without a factory is still refused).
  Under 20 ms each.
Proof: unit, `heavy cargo nextest run -p skein-world-tests --test
  hosted` once; `-p skein-io-world --test cuts` unchanged.
Don't: restart a hosted child from the world; drop a hosted child on a
  kill of another process.
Size: at most 60 lines of production code.

## Interface sketches

```rust
// skein_io::kernel (crates/skein-io/src/kernel.rs): the variants this
// session adds; OpenHow is shown with each variant, as it crosses into the ring, the
// simulator, the conformance suite and smith's matches.

/// How an `Op::Open` opens what its path names.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum OpenHow {
    /// An entry itself, links refused in its parents, for Stat and Close only (kernel.md, 6.1).
    PathNoFollow,
    /// An existing file or directory, to `Read` or `Stat` (`O_RDONLY`).
    Read,
    /// An existing file to read, refusing a symbolic link in any path part.
    ReadNoFollow,
    /// An existing directory: a root beneath this one, or a directory to `List`.
    Directory,
    /// An existing directory, refusing a symbolic link in any path part.
    DirectoryNoFollow,
    /// A new, empty file, to `Write`, made with `mode` less the umask.
    Create { mode: Option<u32> },
    /// A new file, refusing a symbolic link in any parent path part.
    CreateNoFollow { mode: Option<u32> },
    /// An existing regular file, to read and write (`O_RDWR`), never created.
    Write,
    /// An existing file to read and write, refusing a symbolic link in any path part.
    WriteNoFollow,
}
```

`Op` gains `Truncate { fd: Fd, length: u64 }` beside `Sync { fd: Fd }`;
its other variants are unchanged.

```rust
// skein_io::file (crates/skein-io/src/file.rs): the requests and the event
// this session adds; the other variants are unchanged.
use skein_io::kernel::Fd;
use skein_lib::Token;

/// Not a type to write: the three variants 03.2 adds to `file::Request`.
#[derive(PartialEq, Eq, Debug)]
pub enum Added {
    /// Opens an existing regular file to read and write; `Opened` with its length, or `Failed`.
    OpenWrite { owner: Token, root: Fd, name: Box<[u8]> },
    /// As `OpenWrite`, refusing a symbolic link in any path part.
    OpenWriteNoFollow { owner: Token, root: Fd, name: Box<[u8]> },
    /// Sets the open file's length, durable once synced; `Cut`, or `Failed`.
    Cut { owner: Token, file: Token, length: u64 },
}

/// Not a type to write: the variant 03.2 adds to `file::Event`, a `Cut`'s terminal.
#[derive(PartialEq, Eq, Debug)]
pub enum AddedEvent {
    /// The file's length is the one asked for.
    Cut { owner: Token },
}
```

(`Added` and `AddedEvent` are the sketch's way of showing the new
variants with their fields; the code adds them to `file::Request` and
`file::Event`, and no such types exist.)

```rust
// skein_sim::machine (crates/skein-sim/src/machine.rs) and
// skein_fake_machine::fs (testing/skein-fake-machine/src/fs.rs)
use skein_sim::Handle;

/// Not a type to write: the variant 03.1 adds to `skein_sim::Ask`.
#[derive(PartialEq, Eq, Debug)]
pub enum AddedAsk {
    /// The file's length set to `length`: `Reply::Done`.
    Truncate { file: Handle, length: u64 },
}
```

The owner of a file cut keeps its length: io holds no length of its own.
A `Cut` ends in one terminal; a `Truncate` is answered once, never
cancelled.

## Questions to raise when reached

- **Cancelling `Truncate`.** kernel.md, 6.1 names neither `Truncate`
  among the operations that "may be cancelled" nor among those that
  complete "without one". This session treats it as the second (a
  `Cancel` of one is a broken invariant; io waits for it at a deadline).
  On a filesystem that stalls an `ftruncate`, a `Cut` then waits as long
  as the kernel does. The designer decides; changing it later adds the
  simulator's `hung` for it and the conformance pairing a cancel needs.
- **"`Missing`"** (io.md, 5) names no event; this session answers
  `Failed { error: NotFound }`.

## Done when

1. Both backends hold the scenario: `heavy cargo nextest run -p
   skein-conformance-ring -p skein-conformance-sim -E
   'test(a_write_open_finds_what_exists_and_truncate_sets_its_length)'`
   runs two tests and passes.
2. A crash keeps an unsynced cut's old length: `heavy cargo nextest run
   -p skein-fake-machine -E 'test(/a_cut_/)'` passes two tests.
3. The two increments landed through `land`: `git -C ~/src/rust/skein
   log --format='%(trailers:key=Step,valueonly)' origin/main` lists
   `skein-03.1` and `skein-03.2`, each commit's `Gate-Result` naming
   `scripts/check.sh passed`.
