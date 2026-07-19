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

Ordering below is prioritized by what this repo actually needs, not just
reference file order. Two real consumers exist today: the
`tests/integration/lsp.rs` e2e test against the monorepo's own
`jjpwrgem-lsp`, and the external `lsp-bench` benchmark tool (`cargo:lsp-bench`
v0.3.3, run via `mise run bench-lsp-all`, output at `benches/lsp/lsp.md`)
that this whole project is named after. `jjpwrgem-lsp`'s entire real
`LanguageServer` impl (`crates/lsp/src/backend.rs`) is `initialize`/
`did_open`/`did_change`/`did_close`/`formatting`/`code_action`/`shutdown` --
it declares `TextDocumentSyncKind::Full` only (no incremental), implements
no `will_save`/`did_save` hooks, and pushes diagnostics on every doc-sync
call. The external `lsp-bench` tool measures exactly three things per
fixture: `initialize` timing, `diagnostic` (wait-for-diagnostics) timing, and
`formatting` timing + edit count, run against `jjpwrgem-lsp` and
`vscode-json`.

There's a third, more important consumer: **the full reference actually
replaces the external `lsp-bench` tool with a lspresso-powered one.**
`benches/Cargo.toml` there has `lspresso = { path = "../crates/lspresso" }`
as a dev-dependency, and `benches/benches/lsp.rs` is a real criterion
benchmark (`bench_initialize`/`bench_diagnostics`/`bench_formatting`/
`bench_code_action`) driving `jjpwrgem-lsp`, `vscode-json`, and a third
`light-json-lsp` server directly through `Client`, writing an RSS sidecar
alongside criterion's own timing. This is the actual end goal, not the
current external-tool shell-out -- once it lands, `.config/mise/conf.d/
bench.toml`'s `cargo:lsp-bench` tool install and the `lsp-bench -c
"benchmark-${fixture}.yaml"` invocation can go away entirely, replaced by
`cargo bench -p benches --bench lsp`.

Reading that file precisely (not just the external tool's 3-metric yaml)
changes what's actually needed before benches can be migrated. It calls
`capabilities::vscode()` for every client spawn,
`client.did_open(uri, text, LanguageKind::JSON)` and
`client.did_change_full(uri, text)` (the **convenience** signatures --
slice 6, not slices 4/5's raw `protocol_*` methods),
`client.next_diagnostics(&uri)` (slice 4),
`client.supports_formatting()`/`client.formatting(uri, options)` and
`client.supports_code_action()`/`client.code_action(uri, range)` (slice 5),
and `lspresso::parse_uri(...)` (used twice in this file alone, and
throughout the reference's own tests) -- which this branch deleted earlier
in its history on the reasoning that it was "simple and verbose" next to
`path_to_uri`. Worth revisiting given concrete, repeated real-world use in
the reference rather than reverting unilaterally; flagged here rather than
silently restored.

So: diagnostics (4) and formatting/code-action (5) are still the right next
two slices -- they're jjpwrgem-lsp's real capabilities and the external
tool's real metrics -- but slice 6's convenience `did_open`/`did_change_full`
wrappers and `capabilities::vscode()` (currently deferred to "slice 10,
whenever needed" below) are _also_ on the critical path to actually building
the benches replacement, not optional polish. `light_json_lsp_profile`/
`vscode_json_profile`/`jjp_profile` and the sidecar/RSS-writing scaffolding
in `benches/benches/lsp.rs` have no lspresso dependency and can be ported
verbatim whenever the above lands. **Migrating `benches/` off the external
tool and onto this** is real, concrete follow-up work once slices 4, 5, and
slice 6's convenience layer exist -- worth its own tracked task (a
`benches`-scoped commit adding the dev-dependency and porting `lsp.rs`,
matching the "another crate consuming lspresso gets its own commit" pattern
already used for `tests/integration/lsp.rs`), not assumed to happen for
free. Incremental sync/`apply_edits`/save-lifecycle (also in slice 6, but not
called by this benchmark) still have zero real-server payoff in this repo --
worth building for lspresso's general server-agnostic completeness (see
`project_lspresso_standalone`), just not on this critical path.

- **Slice 4 -- diagnostics**: push + pull diagnostics, `wait_for_diagnostics`/
  `wait_for_diagnostics_cleared`, result-id threading
  (`resultid-pull-diagnostics-threaded`), `supports_pull_diagnostics()`. The
  most complex remaining slice -- settle/timeout/pacing logic lives here in
  the reference and the test file bulk (proptest_concurrency.rs-style)
  reflects that. Should also introduce `FakeTransportHandle` (reference
  `client/transport.rs`) + the `proptest` dev-dependency here, not earlier --
  it's an in-memory `Transport` stand-in built specifically so property tests
  over message-ordering/buffering can run fast and deterministically without
  a real subprocess, and nothing before diagnostics/progress has buffering
  complex enough to need it (if that infra turns out to slow this slice down,
  it's fine to land basic push-diagnostics correctness against a real
  fixture first and defer the property-test hardening a step). Update
  `tests/integration/lsp.rs` to actually wait for jjpwrgem-lsp's pushed
  diagnostics once this lands -- it publishes them on every did_open/
  did_change/did_close already. `refresh-client-recomputes`'s `verify` tag
  was dropped from slice 3's `tolerates_and_acknowledges_server_messages`
  test (it wasn't actually proven there) pending this -- the reference
  verifies it via `proptest_concurrency.rs`, not a hand-rolled fixture test;
  re-add the tag once equivalent coverage lands here.
- **Slice 5 -- formatting & code actions**: `formatting`/`range_formatting`/
  `supports_formatting()`, `code_action`/`code_action_resolve`/
  `supports_code_action()`. Pulled out of the language-feature batches below
  and prioritized ahead of them specifically because these are
  `jjpwrgem-lsp`'s only two real request/response capabilities and the only
  two the `lsp-bench`/benchmark suite (`formatting` timing + edit count) and
  `tests/integration/lsp.rs` actually need -- extend that e2e test to call
  `formatting()` on the doc it already opens once this lands. Also where
  position encoding negotiation (`lspclient[posenc-negotiated]`, reference
  `capabilities.rs`'s `build_capabilities`) belongs -- the first slice where
  `Position`/`Range` semantics actually matter to a caller.
- **Slice 6 -- incremental sync, edits & save lifecycle**: `did_open`/
  `did_change_full`/`did_change_incremental`/`did_close` convenience wrappers
  (reclaiming the plain names slice 3 freed up, delegating to the
  `protocol_did_*` macro-generated methods) plus `apply_edits`/`TextEdit`
  application (reference `edits.rs`), negotiating full vs. incremental via
  `text_document_sync_kind()`/`supports_incremental_sync()` -- these two
  belong here, not a separate capabilities slice, since `did_change_full`/
  `did_change_incremental` are the only callers per the reference. Needs
  document-text tracking (`DocumentTracker` growing beyond slice 2/3's bare
  `OpenDocs` `HashSet<Uri>` into an actual `HashMap<Uri, OpenDoc>` with
  version+text) -- first slice that needs to read tracked content back. Also
  where `did_save`/`save_applying_will_save_edits` belong: the reference's
  `save_applying_will_save_edits` calls `did_change_incremental`/
  `did_change_full`/`apply_edits`/`docs.text()` directly, so it's not an
  independent feature -- it's built entirely on this slice's own machinery
  and can't exist before it. Deprioritized behind diagnostics/formatting/
  code-action above since neither real consumer in this repo exercises
  incremental sync or save hooks at all.
- **Slice 7 -- language features, batch 1**: the mechanical `convenience.rs`
  sweep for simple no-special-state requests once slice 3's macro exists --
  `hover`, `completion`, `goto_definition`/`declaration`/`type_definition`/
  `implementation`, `references`, `rename`/`prepare_rename`, `document_symbol`,
  `document_highlight`, `folding_range`, `selection_range`, `signature_help`.
  Each request is close to a one-line `lsp_request!` + wrapper once the macro
  exists. None of these are implemented by `jjpwrgem-lsp` or measured by
  `lsp-bench` -- lowest priority for this repo specifically, still valuable
  for lspresso's general-purpose completeness.
- **Slice 8 -- language features, batch 2 (result-id / resolve pairs)**:
  `semantic_tokens_full`/`_delta`/`_range` (result-id threading, mirrors
  diagnostics' pattern), `code_lens`/`code_lens_resolve`, `document_link`/
  `_resolve`, `inlay_hint`/`_resolve` -- grouped because each pairs a request
  with a `*/resolve` request sharing state. Same low-priority-for-this-repo
  caveat as batch 1.
- **Slice 9 -- progress & cancellation**: `wait_for_progress` (begin/report/end
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

- **Slice 10 -- everything else / cleanup**: file operations (`will*`/`did*`
  create/rename/delete -- these ARE real convenience methods in the
  reference, unlike the workspace-edit items above), notebook documents (if
  in scope at all -- reference explicitly doesn't support them either, see
  the commented-out `lsp_notify!` calls in `protocol.rs`), completion item
  defaults, call hierarchy, moniker, linked editing range, document color,
  inline completion/value, the `helix`/`neovim`/`zed` editor capability
  presets (`capabilities::vscode()` itself is needed earlier -- see the
  benches-migration note above, it's not deferred to this slice), plus the
  trivial `workspace/applyEdit`/`workspace/configuration` decline-answers
  noted above. Reassess whether this is one slice or several once slice 9
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
