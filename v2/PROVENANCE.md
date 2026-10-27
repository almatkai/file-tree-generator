# Pinned evidence

- Own repository: https://github.com/almatkai/file-tree-generator
- Fresh origin/main source: `2d1f4b0bcbf1cfd97e0fe3aa1bdfd0feba18c3a0`.
- Implementation branch: `feat/native-v2-rust`; no commit or push made by this task.
- Original Raycast source: https://github.com/raycast/extensions/tree/6734d85992290066fd31a7f5bee3cca3d3545a1a/extensions/file-tree-generator
- Original author: `csharp` (original package manifest); extension LICENSE copyright 2024 Sungmin Choi.
- Original icon: `assets/command-icon.png`, 169316 bytes,
  SHA256 `6ae8ae29d853b86b840fe54297a2a9d66520de7c7ce86cc206083d6ddfc3add9`.
- Beta contracts: Tezbar `fd300e276356587726566026f072563eb9c34cb1`.

`fixtures/own` contains every tracked own V1 file byte-exact, including command,
internal parser/rendering libraries, package lock, icon, license and tests.
`fixtures/beta` contains byte-exact docs, full schema, common/platform, SDK/runtime,
NativeSessions, native tests (evidence only), shared wire types and renderer files.
No beta source file or dirty app checkout was changed. `fixtures/hashes.json`
records SHA256 for every fixture. The read-only beta LICENSE is copied into
`licenses`. `checks/verify.py` checks own fixtures against the recorded Git pin
when Git is available, and checks the root V1 files remain unchanged (README is
allowed only its appended V2 discovery link). The root V1 icon is also unchanged;
the independently fetched original is the V2 icon.

XERJ prefix `ax-3f837aa7b934`, isolated workspace state, localhost lexical server.
Initial indexing completed generation 1 with all 7/7 code files indexed, exit 3
`completed-with-junk` for non-code assets. This is not a complete index of every
asset. Exact file/hash/visibility checks use Git, SHA256 and rg instead.

Final XERJ reindex was attempted with the same isolated prefix and a 30-second
bound. It aborted immediately on the frozen dataset/schema (new v2/Cargo.toml
family not accepted), so the index is incomplete for V2. No new prefix, global
service restart or shared service mutation was used. Exact checks used rg/Git/hashes.
