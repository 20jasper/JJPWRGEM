# lspresso build plan

`lspresso` is being built up from the full reference implementation (branch
`lsp-bench`) in minimal, independently-reviewable slices, each squashed to one
commit and layered on top of the last on branch `lspresso-slice-1-lifecycle`
(the branch name is stale -- it now carries multiple slices; rename when
convenient). Each slice's `Client` struct holds only the fields that slice's
own methods use -- no fields added speculatively for a future slice.

Workflow per slice: implement -> add fixture binaries + integration tests in
`tests/integration/` -> add spec rules to `spec/lsp-client.md` + `lspclient[impl
...]`/`lspclient[verify ...]` tags -> `cargo build`/`test`/`clippy`/`fmt` ->
`mise run diet` -> commit. Compare against the reference (`git show
lsp-bench:crates/lspresso/src/...`) for fidelity -- match it unless there's a
concrete reason to diverge (call out the reason when it happens).

## Completed

- **Slice 0 -- shell**: crate skeleton, Cargo.toml publish metadata, cargo-rdme
  README generation, `test-binaries` feature. Commit: `74831066`.
- **Slice 1 -- lifecycle**: `spawn`/`spawn_initialized`/`initialize`/`shutdown`/
  `wait_for_exit`, message IO core (`send`/`recv_until`/`await_response`/
  `handle_background`), `Lifecycle`/`PendingRequests` wrapper structs, `trace`
  module, generic `request<R>`/`notify<N>` escape hatch, tolerance for
  server-initiated notifications/requests lspresso doesn't act on. Commit:
  `4991e8f8` (squashed across earlier WIP commits).
- **Slice 2 -- document sync (full-sync only)**: `did_open`/`did_change`/
  `did_close` (raw `gen_lsp_types` params, no convenience wrappers yet),
  `OpenDocs` wrapper enforcing open/close balance, `require_initialized`.
  Deliberately no document-text tracking, no incremental sync, no
  auto-versioning -- nothing in this slice needs to read tracked content back.
  Commit: `f0bac6d3`. Plus a real end-to-end test against `jjpwrgem-lsp` (the
  monorepo's own server, via `jjp lsp`) in the root package's own test suite
  (`tests/integration/lsp.rs`, commit `71770add`) -- not part of the lspresso
  crate itself, but validates it against a real server rather than only
  synthetic fixtures.
- **Slice 3 -- protocol macro system**: `lsp_request!`/`lsp_notify!` macros
  (`pre:`/`post:`/`hook:` arms, matching the reference exactly) centralizing
  the `require_initialized` check so future slices get it automatically
  instead of hand-adding it per method. Renamed slice 2's `did_open`/
  `did_change`/`did_close` to `protocol_did_open_notification`/
  `protocol_did_change_notification`/`protocol_did_close_notification`
  (matching the reference's raw/protocol-layer naming) and refactored them
  onto the macro -- this frees up the plain `did_open`/`did_change`/
  `did_close` names for slice 5's convenience layer, avoiding a second
  breaking rename later. Added `with_uri.rs` (`WithUri` trait, minimal --
  only the 3 params types slice 2/3 actually have, not the reference's full
  set) so `pre: require_open`/`require_not_open` generalize over any future
  params type. Also wired `protocol_set_trace_notification`,
  `protocol_exit_notification`, `protocol_shutdown_request` through the
  macro to prove it out (swapped in for the raw `notify`/`request` escape
  hatch calls two tests were using). Commit: `8fbccf99`.

## Proposed remaining slices

Later boundaries are provisional -- expect them to move as earlier slices land
and clarify what the next one actually needs. Reference file: mostly
`crates/lspresso/src/client/protocol.rs` (1109 lines in the full
implementation) plus `convenience.rs` (629 lines, ~45 thin wrapper methods),
`diagnostics.rs` (307 lines), `edits.rs` (296 lines), `with_uri.rs` (129
lines).

No standalone "capability gating helpers" slice -- the reference's
`capabilities.rs` bundles `supports_formatting`/`supports_pull_diagnostics`/
`supports_code_action`/`text_document_sync_kind`/`supports_incremental_sync`
into one `impl Client` block, but each one only means something once its
paired feature exists (a `supports_formatting()` with no `formatting()` to
gate is dead API, same anti-pattern as slice 3's `protocol_shutdown_request`
finding). `server_capabilities()` itself already shipped in slice 2. Each
helper below is folded into the slice that adds the feature it gates, and
`lspclient[init-capabilities-gate-features]` (removed from `spec/lsp-
client.md` in slice 2) gets re-added piecemeal as each one lands, not all at
once.

Ordering below is prioritized around one explicit goal: **replace the
external `lsp-bench` tool** (`cargo:lsp-bench` v0.3.3, run via `mise run
bench-lsp-all`, output at `benches/lsp/lsp.md`) with a lspresso-powered one,
matching what the full reference actually does. `benches/Cargo.toml` there
has `lspresso = { path = "../crates/lspresso" }` as a dev-dependency, and
`benches/benches/lsp.rs` is a real criterion benchmark
(`bench_initialize`/`bench_diagnostics`/`bench_formatting`/
`bench_code_action`) driving `jjpwrgem-lsp`, `vscode-json`, and a third
`light-json-lsp` server directly through `Client`, writing an RSS sidecar
alongside criterion's own timing. The `tests/integration/lsp.rs` e2e test
against `jjpwrgem-lsp` is the other real consumer and wants the same things.

This doesn't have to land as one big migration at the end -- each of the
four `bench_*` functions has its own, smaller dependency chain, and three of
them only need the **raw** `protocol_did_open_notification`/
`protocol_did_change_notification` methods (already shipped, slice 3) with a
version number the benchmark code tracks itself, not slice 7's convenience
`did_open`/`did_change_full` wrappers or any document-tracking
infrastructure at all. So each slice below should land its own small,
incremental `benches/`-scoped commit porting whatever `bench_*` function it
just unblocked, same pattern as `tests/integration/lsp.rs` (its own
"test:"-prefixed commit, colocated right after the slice it depends on) --
not one deferred migration after slice 6 or 7:

- **Now, right after slice 1** (already shipped): port `bench_initialize`
  using `ClientCapabilities::default()` as a stand-in for
  `capabilities::vscode()`. `bench_initialize` only calls `Client::spawn`/
  `.initialize(...)`/`.shutdown()`/`.rss_bytes()`, all slice-1 surface --
  slice 4 isn't actually a dependency, just the source of the "real" value
  to swap in once it exists. Checked whether `default()` vs `vscode()` would
  produce different runtime behavior against the profiles this benchmark
  spawns: `jjpwrgem-lsp`'s `initialize` only reads
  `capabilities.general.position_encodings` to decide UTF8 vs UTF16 (see
  `crates/lsp/src/backend.rs`), which is `None` under both `default()` and
  `vscode()` (the latter declares only `PositionEncodingKind::UTF16`, i.e.
  no UTF8) -- so for every profile this benchmark can actually reach today,
  the two are behaviorally identical. One-line swap once slice 4 lands.
  Landed as its own `benches/`-scoped commit (`benches/Cargo.toml` gains the
  `lspresso` dev-dependency + `[[bench]] name = "lsp"`; new
  `benches/benches/lsp.rs` with just `Profile`/`Sidecar` (`initialize` field
  only)/`profiles()`/`bench_initialize`), following the same "own commit,
  colocated right after the slice it depends on" pattern as
  `tests/integration/lsp.rs`.
- After **slice 5** (diagnostics): port `bench_diagnostics` -- needs
  `client.next_diagnostics(&uri)` plus doc-open/doc-change, which can use
  the already-shipped raw `protocol_did_open_notification`/
  `protocol_did_change_notification` with a benchmark-local version
  counter, not slice 7.
- After **slice 6** (formatting & code actions): port `bench_formatting` and
  `bench_code_action` -- same raw-doc-sync approach. **This is the point
  where all four benchmarks are ported and the external tool can be fully
  removed** (`.config/mise/conf.d/bench.toml`'s `cargo:lsp-bench` install +
  `lsp-bench -c "benchmark-${fixture}.yaml"` invocation deleted, replaced by
  `cargo bench -p benches --bench lsp`) -- a full slice earlier than treating
  slice 7 as a prerequisite would have gotten there.
- After **slice 7** (convenience doc-sync): no longer a blocker for anything
  above, since it landed after the tool was already fully replaced. It's a
  cleanup pass on the now-working benches code (swap the raw calls +
  benchmark-local version counter for the nicer `did_open(uri, text, lang)`/
  `did_change_full(uri, text)` signatures, matching the reference
  byte-for-byte) plus the same simplification for `tests/integration/lsp.rs`.
  Still worth doing soon since it's small, just not gating the main goal.

Reading `benches/benches/lsp.rs` precisely (not just the external tool's
3-metric yaml) is what the dependency chains above are built from. It also
calls `lspresso::parse_uri(...)` (used twice in this file alone, and
throughout the reference's own tests) -- which this branch deleted earlier
in its history on the reasoning that it was "simple and verbose" next to
`path_to_uri`. Worth revisiting given concrete, repeated real-world use in
the reference rather than reverting unilaterally; flagged here rather than
silently restored. `jjpwrgem-lsp`'s entire real `LanguageServer` impl
(`crates/lsp/src/backend.rs`) is `initialize`/`did_open`/`did_change`/
`did_close`/`formatting`/`code_action`/`shutdown` -- declares
`TextDocumentSyncKind::Full` only (no incremental), implements no
`will_save`/`did_save` hooks, and pushes diagnostics on every doc-sync call.
Neither real consumer needs incremental sync, `apply_edits`, or save
lifecycle at all, so that's pushed past everything the benches migration
needs, not ahead of it.

- **Slice 4 -- capability presets**: `capabilities::vscode()` only (defer
  `helix`/`neovim`/`zed` -- nothing in this repo calls them). Pure data
  construction (`build_capabilities`/`build_pull_diagnostics`, a
  `ClientCapabilities` value with `PositionEncodingKind::UTF16` and pull-
  diagnostics support declared) with zero `Client`-state dependency -- it
  doesn't need diagnostics/formatting/anything else to exist first, it's
  just what every later slice's tests and the benches migration should be
  spawning with instead of `ClientCapabilities::default()`. Cheapest
  possible slice; moved ahead of everything else specifically because every
  other slice's tests benefit from using a realistic capabilities value
  from here on. Swap `benches/benches/lsp.rs`'s `ClientCapabilities::default()`
  stand-in for `capabilities::vscode()` here (see the "right after slice 1"
  entry above -- `bench_initialize` itself already ported).
- **Slice 5 -- diagnostics**: push + pull diagnostics, `wait_for_diagnostics`/
  `wait_for_diagnostics_cleared`/`next_diagnostics`, result-id threading
  (`resultid-pull-diagnostics-threaded`), `supports_pull_diagnostics()`. The
  most complex remaining slice -- settle/timeout/pacing logic lives here in
  the reference and the test file bulk (proptest_concurrency.rs-style)
  reflects that. Checked whether this could be scoped down to just
  push-diagnostics (all `jjpwrgem-lsp` needs) to shrink this slice, but the
  reference's `diagnostics_once`/`next_diagnostics` unconditionally branch
  on `supports_pull_diagnostics()` and share result-id caching with the pull
  path -- splitting push out cleanly would diverge from the reference's
  actual structure for no strong reason, so keep it as one slice. Should
  also introduce `FakeTransportHandle` (reference `client/transport.rs`) +
  the `proptest` dev-dependency here, not earlier -- an in-memory
  `Transport` stand-in built specifically so property tests over
  message-ordering/buffering can run fast and deterministically without a
  real subprocess, and nothing before diagnostics/progress has buffering
  complex enough to need it (if that infra turns out to slow this slice
  down, it's fine to land basic push-diagnostics correctness against a real
  fixture first and defer the property-test hardening a step). Update
  `tests/integration/lsp.rs` to actually wait for jjpwrgem-lsp's pushed
  diagnostics once this lands. `refresh-client-recomputes`'s `verify` tag
  was dropped from slice 3's `tolerates_and_acknowledges_server_messages`
  test (it wasn't actually proven there) pending this -- the reference
  verifies it via `proptest_concurrency.rs`, not a hand-rolled fixture test;
  re-add the tag once equivalent coverage lands here. Port `bench_diagnostics`
  right after (see above).
- **Slice 6 -- formatting & code actions**: `formatting`/`range_formatting`/
  `supports_formatting()`, `code_action`/`code_action_resolve`/
  `supports_code_action()`. Pulled out of the language-feature batches below
  and prioritized ahead of them specifically because these are
  `jjpwrgem-lsp`'s only two real request/response capabilities, the only two
  `lsp-bench`/the benchmark suite need, and both are called directly in
  `benches/benches/lsp.rs`. Port `bench_formatting`/`bench_code_action` right
  after -- **the external tool can be fully removed once this lands** (see
  above).
- **Slice 7 -- convenience doc-sync**: `did_open(uri, text, lang)`/
  `did_change_full(uri, text)`/`did_close(uri)`, reclaiming the plain names
  slice 3 freed up and delegating to the `protocol_did_*` macro-generated
  methods. `did_change_full` needs `docs.next_version(uri)` (bare
  version-number tracking, a much smaller lift than full text tracking) and
  `text_document_sync_kind()` (a plain read of the already-stored
  `server_capabilities()`, not real negotiation logic) -- both scoped
  minimally here, not the full `DocumentTracker` upgrade. No longer gates
  the benches migration (that finished at slice 6) -- this is where the
  already-working `benches/benches/lsp.rs`/`tests/integration/lsp.rs` code
  gets simplified to drop their benchmark/test-local version counters in
  favor of these wrappers, matching the reference exactly.
- **Slice 8 -- incremental sync, edits & save lifecycle**: `did_change_incremental`
  plus `apply_edits`/`TextEdit` application (reference `edits.rs`),
  negotiating full vs. incremental via `supports_incremental_sync()`. Needs
  the full `DocumentTracker` upgrade slice 7 deliberately deferred (bare
  version tracking growing into an actual `HashMap<Uri, OpenDoc>` with
  version+text) -- first slice that needs to read tracked content back. Also
  where `did_save`/`save_applying_will_save_edits` belong: the reference's
  `save_applying_will_save_edits` calls `did_change_incremental`/
  `did_change_full`/`apply_edits`/`docs.text()` directly, so it's not an
  independent feature -- it's built entirely on this slice's own machinery
  and can't exist before it. Deprioritized behind the whole benches-migration
  path above since neither real consumer in this repo exercises incremental
  sync or save hooks at all.
- **Slice 9 -- language features, batch 1**: the mechanical `convenience.rs`
  sweep for simple no-special-state requests once slice 3's macro exists --
  `hover`, `completion`, `goto_definition`/`declaration`/`type_definition`/
  `implementation`, `references`, `rename`/`prepare_rename`, `document_symbol`,
  `document_highlight`, `folding_range`, `selection_range`, `signature_help`.
  Each request is close to a one-line `lsp_request!` + wrapper once the macro
  exists. None of these are implemented by `jjpwrgem-lsp` or measured by
  `lsp-bench` -- lowest priority for this repo specifically, still valuable
  for lspresso's general-purpose completeness. Also where full position
  encoding negotiation (`lspclient[posenc-negotiated]`, consuming what
  slice 4's `vscode()` preset declares) matters for the first time --
  the first slice where `Position`/`Range` semantics actually matter to a
  caller.
- **Slice 10 -- language features, batch 2 (result-id / resolve pairs)**:
  `semantic_tokens_full`/`_delta`/`_range` (result-id threading, mirrors
  diagnostics' pattern), `code_lens`/`code_lens_resolve`, `document_link`/
  `_resolve`, `inlay_hint`/`_resolve` -- grouped because each pairs a request
  with a `*/resolve` request sharing state. Same low-priority-for-this-repo
  caveat as batch 1.
- **Slice 11 -- progress & cancellation**: `wait_for_progress` (begin/report/end
  aggregation, `ProgressToken` uniqueness), `cancel_request`
  (`$/cancelRequest`, `cancel-still-responds`), `work_done_progress_cancel`.

No "workspace edits & command execution" slice -- checked the reference and
neither is actually implemented there. `workspace/applyEdit` is declined:
`answer_server_request` answers it with `ApplyWorkspaceEditResult { applied:
false, .. }` and a `tracing::warn!`, and the real client-initiated
`apply_workspace_edit` is commented out in `protocol.rs` with a note that
correct application needs overlap/version validation lspresso doesn't have.
`workspace/executeCommand` is commented out too, never built at all. Same
story for a few things previously listed under "everything else":
**dynamic registration** is already fully done, as of slice 1 --
`RegistrationRequest`/`UnregistrationRequest` are already in the generic
null-answer group in `answer_server_request`, matching the reference exactly
(it never tracks registration ids or implements `dynreg-unregister-same-id`/
`dynreg-honor-until-unregistered`/`dynreg-scoped-by-document-selector` beyond
that either) -- nothing to build, just optionally add the spec rules as
documentation for coverage tracking. **File watching**
(`DidChangeWatchedFilesNotification`) is never built in the reference,
permanently out of scope, not deferred. **`workspace/configuration`** is the
same tiny declined-with-an-explicit-answer pattern as `applyEdit` (answers
each requested item `null`) -- a few lines, addable whenever convenient
alongside any slice already touching `answer_server_request`, not real
feature work.

- **Slice 12 -- everything else / cleanup**: file operations (`will*`/`did*`
  create/rename/delete -- these ARE real convenience methods in the
  reference, unlike the workspace-edit items above), notebook documents (if
  in scope at all -- reference explicitly doesn't support them either, see
  the commented-out `lsp_notify!` calls in `protocol.rs`), completion item
  defaults, call hierarchy, moniker, linked editing range, document color,
  inline completion/value, the `helix`/`neovim`/`zed` editor capability
  presets (`capabilities::vscode()` itself already shipped in slice 4), plus
  the trivial `workspace/applyEdit`/`workspace/configuration` decline-answers
  noted above. Reassess whether this is one slice or several once slice 11
  lands -- likely fewer than previously estimated now that dynamic
  registration/file watching/command execution are confirmed either
  already-done or permanently out of scope.

## Notes for whoever picks this up

- lspresso is a standalone, server-agnostic library (see project memory
  `project_lspresso_standalone`) -- never add a dependency on another
  monorepo crate to lspresso itself. The reverse direction (another crate
  depending on lspresso as a dev-dependency for its own tests, as
  `jjpwrgem`'s root package now does) is fine and encouraged.
- Client-side spec verification only: lspresso's own tests must exercise
  lspresso's behavior, not real-server/`gen_lsp_types` compliance.
- Comments: extremely sparse by design (see prior review rounds this repo has
  been through) -- only where a WHY is genuinely non-obvious. Public
  functions that can panic get a `# Panics` doc comment; everything else
  documents fallibility via `Result`/`Option` in the signature.
- When a review finding turns out to match the reference byte-for-byte, it's
  not a bug to fix -- verify against `git show lsp-bench:<path>` before
  "fixing" anything a review flags, since several false positives this way
  have already been caught (duplicate `didopen-max-one-open`/
  `didopen-close-open-balanced` spec rules, an "unused" `AlreadyShutdown`
  test path, a "too-loose" `uri-consistent` test) -- the reference itself
  has some accepted rough edges that are fine to carry forward faithfully.
  `uri-identifies-document` is the same story: `impl`-tagged only, never
  `verify`-tagged anywhere in the mature reference either -- don't add a
  `verify` tag for it unless a test genuinely asserts on the `uri` field
  itself (checked and reverted once already, see slice 3's commit history).
