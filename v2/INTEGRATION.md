# Compiled beta integration (not performed)

Target contracts: Tezbar beta `fd300e276356587726566026f072563eb9c34cb1`.
The dirty `fix/deep-search-state` app checkout has only been read with `git show`.
No registration, catalog, profile or installed-database edits were performed.

1. Vendor this entire `v2` package at the stable location
   `src-tauri/native-extensions/file-tree-generator`. Retain its licenses,
   provenance, Cargo.lock and fixtures. Do not vendor into a temporary clone path.
2. Copy `adapter/v2_file_tree_generator.rs` to
   `src-tauri/src/native_core/extensions/v2_file_tree_generator.rs`.
   Its sibling SDK import is `super::v2_sdk`. Its engine path is
   `../../../native-extensions/file-tree-generator/src/lib.rs`. There are no
   absolute paths or paths back to the temporary repository. The host requires
   serde (derive), serde_json and uuid (v4), already available in pinned beta.
   The standalone crate is independently locked for Rust 1.85 and stable.
3. Declare `mod v2_file_tree_generator;` in the extensions module. Update the
   compiled `v2_runtime.rs` dispatch:

   ```rust,ignore
   // slug's accepted-slug match:
   "file-tree-generator" => Some(value),
   // commands:
   "file-tree-generator" => super::v2_file_tree_generator::commands(),
   // async handle:
   "file-tree-generator" => {
       super::v2_file_tree_generator::handle(ctx, command, operation, payload, state).await
   }
   ```

   Keep existing handlers. `slug` already strips canonical and `raycast.`
   prefixes. `commands()` returns exactly `["index"]`; `handle` accepts the
   unchanged beta signature (`CoreContext`, command, operation, payload, state).
   Include the bundled `manifest.json` object in `v2_catalog.json`. JSON cannot
   load Rust: without compilation/registration it must report unavailable.
4. Copy the exact `assets/command-icon.png` to
   `src/renderer/public/extensions/icons/file-tree-generator.png`. The manifest
   URL must remain `/extensions/icons/file-tree-generator.png`. SHA256 must be
   `6ae8ae29d853b86b840fe54297a2a9d66520de7c7ce86cc206083d6ddfc3add9`.
   Do not add unsupported root `author` metadata; credits are in description,
   notices and provenance. Retain V1 as the reversible fallback.
5. Extend catalog/coverage tests: canonical and replaced ID resolve to this slug,
   `index` is compiled on both declared targets, all manifest command names equal
   compiled `commands()`, unknown commands remain unavailable, installed-command
   checks are enforced, metadata describes no arguments/preferences, and both
   verification records remain `not-run`. Validate the full pinned schema.
   Rebuild the host and use its actual compiled coverage CLI to inspect results.
   Do not mark support or QA from static manifest presence.
6. Install via normal canonical Native V2 managed installation. The native marker
   cannot provide execution on its own. No unknown legacy storage conversion is
   implemented. The native SDK namespace is `native-extensions-v2/file-tree-generator`.

Actual wire operations are `extension:run-command`, `extension:invoke-action`,
`extension:search-text-changed`, `extension:refresh-session`,
`extension:load-more`, `extension:dispose-session`. Launch has no arguments or
preferences in V1; `payload.argumentValues` and `payload.preferences` are ignored.
Fields retain their V1 IDs. `actionId` is a session-bound opaque capability;
field callbacks use `{formValues:{value}}`, and the submit-form action uses the
complete form map. Caller-supplied output is ignored and never used as copy text.
No list search is exposed, so server search/filter flags are unnecessary.

Only SDK `clipboard:write[text]` is used. Its actual object acknowledgement is
strictly checked. No real clipboard/selection/window/focus/paste/browser action
was performed in validation. In particular the pinned `window:hide[]` returns
null after hiding; a future hide action requires a host acknowledgement fix.
Do not normalize arbitrary null responses into success. The form stays open and
uses supported Form.Description feedback; it does not advertise arbitrary
shortcuts or unsupported HUD/toast/pop-to-root behavior.

`checks/materialize.py` builds an extracted, self-contained headless contract
crate. It compiles unchanged `common.rs`, SDK and `NativeSessions` as a dependency
with labeled Tauri/registry/installation/compiled-registry stubs. Its root also
compiles and lints the exact production adapter against the unchanged SDK types.
The full beta `native_tests.rs` is retained as evidence, not executed: it covers
unrelated host modules outside this package. Our 15 integration tests drive actual
NativeSessions and SDK through their public paths. Renderer checks execute actual
Form/List/Detail parsers, reconciliation and surface routing; React/DOM and
unrelated UI imports are stubbed. This does not establish full-host build success.

Host registration/rebuild and real macOS/Windows UI, clipboard, focus and paste
QA remain **not-run**. No paste/focus feature is introduced by this port.
