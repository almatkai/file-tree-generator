# Notices and licensing evidence

V2 is a modified Rust translation of `almatkai/file-tree-generator` at
`2d1f4b0bcbf1cfd97e0fe3aa1bdfd0feba18c3a0`, which derives from the Raycast extension
by **csharp** at `6734d85992290066fd31a7f5bee3cca3d3545a1a`.
The extension LICENSE in both pinned sources is byte-identical Apache-2.0,
Copyright 2024 Sungmin Choi. Both package manifests declare MIT instead. This
conflict is unresolved upstream; this port follows the actual extension license
text and retains the conflicting manifests as evidence. It does not assert that
MIT overrides that file. Rust files carry modification notices. The exact
original icon is retained, without an assertion of separate trademark rights.

The Raycast repository root LICENSE is MIT, Copyright (c) 2021 Raycast; the
extension-specific Apache text is more specific to the ported implementation.
The pinned Tezbar beta root LICENSE is Apache-2.0. Both texts are included.
No missing extension license was inferred from a package name or SPDX field.

The distributed package and its build/runtime dependency notices use the
aggregate SPDX expression **Apache-2.0 AND MIT AND Unicode-3.0**. The port itself
is Apache-2.0. For dual-licensed Rust dependencies, MIT is selected where offered;
ryu uses its Apache-2.0 alternative, and unicode-ident additionally requires
Unicode-3.0. `licenses/rust-inventory.json` records exact locked versions, declared
expressions, selected alternatives, and copied actual license texts. It includes
all lock targets (including untested WASI/web dependencies), not a claim that all
are compiled on macOS/Windows. `licenses/hashes.json` authenticates these copies.

Validation fixtures for the locked lodash 4.17.23, tsx 4.23.13, esbuild 0.27.7
and React are MIT, with their actual package manifests/license texts retained
under `fixtures/locked`. The locked tsx dist and esbuild JS are validation
fixtures, not native runtime fallbacks. Own TS parser/renderer import only their
own modules, and never execute lodash or @raycast/utils. Oracle expectations come
from those actual TS modules; the actual TSX form runs with React/Raycast boundary
stubs. npm dependencies are installed from the byte-exact own package-lock for
validation only. The npm dependency tree is not vendored as a product runtime.
