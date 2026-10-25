#!/usr/bin/env python3
"""Package inventory and extracted offline checks; writes only beneath this package."""
from pathlib import Path
import hashlib,json,subprocess,tarfile,sys,os
v=Path(__file__).resolve().parents[1]
out=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else v/'checks/materialized-package-final'
if out.exists(): raise SystemExit('Package extraction directory already exists; choose a fresh suffix in this validation script')
out.mkdir()
def run(args, name):
 with (out/(name+'.log')).open('w') as log:
  subprocess.run(args,cwd=v,check=True,stdout=log,stderr=subprocess.STDOUT)
 print('PASS',name,flush=True)
run(['cargo','package','--manifest-path',str(v/'Cargo.toml'),'--allow-dirty','--offline','--locked'],'cargo-package')
archive=v/'target/package/tezbar-file-tree-generator-0.2.0.crate'
with tarfile.open(archive) as tar:
 names=tar.getnames();prefix=names[0].split('/')[0]+'/'
 assert not any('node_modules' in n or 'materialized' in n for n in names)
 required=['src/lib.rs','adapter/v2_file_tree_generator.rs','assets/command-icon.png','manifest.json','INTEGRATION.md','VALIDATION.md','checks/templates/Cargo.toml.in','checks/templates/fixture-Cargo.toml.in','checks/templates/tauri/Cargo.toml.in']
 for p in required: assert prefix+p in names,p
 for p in json.loads((v/'fixtures/hashes.json').read_text()): assert prefix+p in names,p
 for p in json.loads((v/'licenses/hashes.json').read_text()): assert prefix+p in names,p
 for member in tar.getmembers():
  assert not member.issym() and not member.islnk(); assert os.path.commonpath([str(out.resolve()),str((out/member.name).resolve())])==str(out.resolve())
 tar.extractall(out)
extracted=out/prefix.rstrip('/')
for toolchain in ['stable','1.85.0']:
 run(['cargo','+'+toolchain,'test','--manifest-path',str(extracted/'Cargo.toml'),'--offline','--locked'],'extracted-core-'+toolchain)
contract=out/'contract'
run([sys.executable,str(extracted/'checks/materialize.py'),str(contract)],'extracted-materialize')
for toolchain in ['stable','1.85.0']:
 run(['cargo','+'+toolchain,'test','--manifest-path',str(contract/'Cargo.toml'),'--offline','--locked'],'extracted-adapter-'+toolchain)
print('PASS inventory',len(names),'archive entries; SHA256',hashlib.sha256(archive.read_bytes()).hexdigest())
