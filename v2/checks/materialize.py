#!/usr/bin/env python3
"""Materialize a self-contained crate after packaging (nested Cargo crates are omitted)."""
from pathlib import Path
import shutil,sys
v=Path(__file__).resolve().parents[1]
out=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else v/'checks/materialized'
if out.exists():
    raise SystemExit('Destination exists; choose a fresh directory')
shutil.copytree(v/'checks/templates',out)
for path in out.rglob('Cargo.toml.in'): path.rename(path.with_name('Cargo.toml'))
(out/'src/native_core/extensions').mkdir(parents=True)
(out/'fixture/src/native_core/extensions').mkdir(parents=True)
shutil.move(out/'fixture-lib.rs.in',out/'fixture/src/lib.rs')
shutil.move(out/'fixture-Cargo.toml.in',out/'fixture/Cargo.toml')
# The dependency's relative Tauri path resolves at fixture/tauri.
shutil.copytree(out/'tauri',out/'fixture/tauri')
shutil.move(out/'lib.rs',out/'src/lib.rs')
shutil.copy2(v/'manifest.json',out/'manifest.json')
base=v/'fixtures/beta/src-tauri/src/native_core'
for file in ['common.rs','extensions/v2_sdk.rs','extensions/native.rs']:
    shutil.copy2(base/file,out/'fixture/src/native_core'/file)
for root in [out, out/'fixture']:
    shutil.copy2(v/'adapter/v2_file_tree_generator.rs',root/'src/native_core/extensions/v2_file_tree_generator.rs')
    dest=root/'native-extensions/file-tree-generator/src';dest.mkdir(parents=True)
    shutil.copy2(v/'src/lib.rs',dest/'lib.rs')
lock=v/'checks/host-Cargo.lock'
if lock.exists(): shutil.copy2(lock,out/'Cargo.lock')
print(out)
