# File Tree Generator — Rust Native V2

An offline Rust library and compiled-in Tezbar adapter alongside unchanged V1.
Generates tree **text** from indentation; it never creates filesystem trees or
interprets names as paths. Original Raycast author: csharp.

`manifest.json` replaces `raycast.file-tree-generator` with
`com.tezbar.native.file-tree-generator`, command `index` / **Create File Tree**.
There are no launch arguments, preferences, source-priority reads or history in
the actual own command. All six original fields remain: Input, Charset, Show
Trailing /, Show Full Path, Show Root ., and computed Output. Copy Output copies
the computed text. Output edits are ignored, as in V1. Only Charset, Full Path
and Root . persist, matching `storeValue`; input and trailing slash reset on a
new session. No legacy storage is migrated.

The library `generate(input, &Options)` has V1 library defaults; use
`Options::form_defaults()` for command defaults. Input order, duplicate names,
bullets, JS whitespace, CR/LF splitting, tabs, literal slashes, Unicode and
ASCII/box characters follow the own TS. A 1 MiB input, 8192 nodes, depth 256,
and 8 MiB output cap return explicit errors instead of truncating. Native JSON
requires valid Unicode scalar values; unpaired JS UTF-16 surrogates are outside
that wire boundary. No input filesystem access, network, JS, Bun or subprocess
is used at product runtime. SDK atomic storage is the only product file write.

The beta's Form wire format supports the preserved form directly. Stable field
IDs and opaque per-session capabilities handle queued drafts; copy capabilities
bind to committed output and rotate on edits. Submit receives the full map;
field changes receive `formValues.value`. Unrelated controlled values are omitted
on acknowledgements so pending local edits survive. Search, refresh and load-more
return unchanged without allocating IDs or changing drafts. There is no additional
navigation screen; host Back exits the form. No hotkeys are advertised.

Copy uses SDK `clipboard:write[text]` and requires an object with `ok:true` and
`success` absent/true, with `error` absent/null. Transport and malformed responses show a
sanitized error while preserving output. Retry is explicit; refresh never repeats
copy. The form remains open with a status description, an honest native UI
adaptation of Raycast feedback. No HUD/toast/pop-to-root API is invented. Hide,
paste, selection, clipboard read and browser routes are not needed by this source.
The pinned `window:hide` returns null: it would fail strict acknowledgement; a
host return-contract fix is required before any future hide action can use it.

See [INTEGRATION.md](INTEGRATION.md), [VALIDATION.md](VALIDATION.md),
[PROVENANCE.md](PROVENANCE.md) and [NOTICE.md](NOTICE.md).
