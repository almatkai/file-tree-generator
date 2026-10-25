# Validation

The native runtime is pure offline Rust. Node/Python are validation-only.
All OS effects use recording/failing routers. No real clipboard, selection,
window, focus, paste or browser operation was executed.

Recorded environment: macOS arm64, Node 26.7.0, stable Rust 1.98.0, Rust 1.85.0.
Standalone and production-adapter contract checks use locked Cargo dependencies.

- Core: 4 tests (including all resource-limit classes and output overflow).
- Contract crate: the same 4 embedded engine tests plus 15 adapter/actual SDK/
  actual NativeSessions integration tests per toolchain. Tests cover full-map
  submit, field callbacks, queued drafts, forged output, malformed calls,
  capability isolation/disposal/staleness, mutation-free refresh/search/load-more,
  SDK atomic concurrent transactions and unknown keys, corruption preservation,
  save-failure draft/output checkpointing and strict effect acknowledgement matrix.
  Valid JSON containing corrupt saved settings is preserved during an in-flight
  session update; explicit success with a null error field is accepted.
- Oracle: 8592 actual own TS parse/render → Rust comparisons; all 16 options,
  537 curated/deterministic generated inputs. Actual own TSX form state/actions
  and storeValue flags execute with only React/Raycast boundary stubs. Locked
  package manifests are checked against retained library fixtures. Own parse and
  render import no external runtime library; lodash/utils are not invoked.
- Renderer: 18 assertions against actual pinned Form/List/Detail parsers, form reconciliation and
  surface routing execute with React/DOM/unrelated UI boundaries stubbed.
  Covers omitted controlled values retaining queued drafts and full-map submit.
- Stable and 1.85: fmt and strict all-target Clippy, core/adapter tests, Windows
  GNU all-target check and strict Clippy. This is compilation, not Windows execution.
- Full pinned draft-2020-12 schema validation and 17 negative manifests; exact
  fixture/license/icon SHA256 and root V1 preservation; required fixture Git visibility.
- Own V1 `npm test`: 1 smoke test passed; `npm run typecheck` passed inside the
  disposable `fixtures/own` installation, with root source/lock untouched.
- Cargo package includes the original icon, locked library dist fixtures and
  Cargo.toml.in templates. Extracted library and materialized adapter tests run
  offline against the populated local Cargo cache. npm dependencies and Rust
  registry sources are not included wholesale: a first clean-machine validation
  needs the locked dependency download/cache. Product runtime needs no cache.

## Reproduce from repository root

```sh
npm ci --prefix v2/fixtures/own --ignore-scripts --no-audit --no-fund
npm test --prefix v2/fixtures/own
npm run typecheck --prefix v2/fixtures/own
cargo build --manifest-path v2/Cargo.toml --locked --example oracle_bridge
node v2/checks/oracle.mjs
python3 v2/checks/run-rust.py /fresh/absolute/contract-directory
FILE_TREE_WIRE_OUT=/fresh/absolute/contract-directory/wire.json node v2/checks/renderer.mjs
python3 -m venv /fresh/absolute/validation-venv
/fresh/absolute/validation-venv/bin/pip install -r v2/checks/requirements.txt
/fresh/absolute/validation-venv/bin/python v2/checks/verify.py
cargo package --manifest-path v2/Cargo.toml --allow-dirty --locked --offline
```

`run-rust.py` refuses an existing destination. It assumes both Rust toolchains
and their `x86_64-pc-windows-gnu` standard libraries have been installed and
locked dependencies are cached. It checks byte-exact common/SDK/native copies.
`cargo fmt --manifest-path <generated>/Cargo.toml --check` checks generated owned
adapter/engine/tests; beta fixtures are not edited to make them pass. To inspect
packaging, `cargo package --list` must contain no node_modules/materialized paths.
Extract the `.crate` to a fresh directory, run `cargo test --offline --locked`
there, then run `python3 checks/materialize.py <fresh-contract-dir>` and
`cargo test --manifest-path <fresh-contract-dir>/Cargo.toml --offline --locked`.
Nested Cargo.toml files are deliberately shipped as `.in` templates because
Cargo packaging omits nested packages. The materializer restores their names.

Initial XERJ: all 7 own code files indexed; assets marked junk, so not an exhaustive
asset index. Git/hash checks provide exact preservation evidence.

## Limits and not-run work

Host registration/catalog/coverage changes and rebuild were not performed.
Full beta app/UI tests were not run. Real macOS and Windows UI/clipboard/focus/
paste QA remains `not-run` in the manifest. Linux/WASI/web platforms are not
claimed; their optional locked dependency branches are not MSRV-verified.
The original extension Apache LICENSE conflicts with MIT package metadata;
that upstream evidence conflict remains unresolved (see NOTICE.md).

Final XERJ reindex was attempted with the same isolated prefix and a 30-second
bound. It aborted immediately on the frozen dataset/schema (new v2/Cargo.toml
family not accepted), so the index is incomplete for V2. No new prefix, global
service restart or shared service mutation was used. Exact checks used rg/Git/hashes.
