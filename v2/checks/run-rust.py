#!/usr/bin/env python3
"""Run locked offline Rust gates against a fresh generated contract crate."""
from pathlib import Path
import subprocess,sys,os
v=Path(__file__).resolve().parents[1]
out=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else v/'checks/materialized-final'
subprocess.run([sys.executable,str(v/'checks/materialize.py'),str(out)],check=True)
def run(*args): subprocess.run(args,check=True,cwd=v,env={**os.environ,'FILE_TREE_WIRE_OUT':str(out/'wire.json')})
run('cargo','fmt','--manifest-path',str(v/'Cargo.toml'),'--check')
run('cargo','fmt','--manifest-path',str(out/'Cargo.toml'),'--check')
for toolchain in ['stable','1.85.0']:
 for manifest in [v/'Cargo.toml',out/'Cargo.toml']:
  run('cargo','+'+toolchain,'test','--manifest-path',str(manifest),'--locked','--offline')
  run('cargo','+'+toolchain,'clippy','--manifest-path',str(manifest),'--all-targets','--locked','--offline','--','-D','warnings')
  run('cargo','+'+toolchain,'check','--manifest-path',str(manifest),'--all-targets','--target','x86_64-pc-windows-gnu','--locked','--offline')
  run('cargo','+'+toolchain,'clippy','--manifest-path',str(manifest),'--all-targets','--target','x86_64-pc-windows-gnu','--locked','--offline','--','-D','warnings')
for relative in ['common.rs','extensions/v2_sdk.rs','extensions/native.rs']:
 assert (out/'fixture/src/native_core'/relative).read_bytes()==(v/'fixtures/beta/src-tauri/src/native_core'/relative).read_bytes()
print('PASS stable + 1.85 tests, strict all-target Clippy, Windows GNU check/Clippy; fixtures unchanged')
