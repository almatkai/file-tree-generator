#!/usr/bin/env python3
from pathlib import Path
import json,hashlib,copy,subprocess
from jsonschema import Draft202012Validator
v=Path(__file__).resolve().parents[1]
schema=json.loads((v/'fixtures/beta/schemas/extension-v2.schema.json').read_text());Draft202012Validator.check_schema(schema);validator=Draft202012Validator(schema)
m=json.loads((v/'manifest.json').read_text());validator.validate(m)
negative=[]
for key,value in [('author','unsupported'),('runtime','javascript'),('id','raycast.file-tree-generator'),('icon','assets/icon.png'),('schemaVersion',1),('commands',[]),('replaces',[])]:
 n=copy.deepcopy(m);n[key]=value;negative.append(n)
for key in schema['required']:
 n=copy.deepcopy(m);del n[key];negative.append(n)
for n in negative: assert not validator.is_valid(n)
assert m['commands'][0]['name']=='index' and m['preferences']==[] and m['commands'][0]['arguments']==[]
assert m['replaces']==['raycast.file-tree-generator']
assert len(m['verification'])==2 and all(x['state']=='not-run' for x in m['verification'])
for path,expected in json.loads((v/'fixtures/hashes.json').read_text()).items():
 assert hashlib.sha256((v/path).read_bytes()).hexdigest()==expected,path
for path,expected in json.loads((v/'licenses/hashes.json').read_text()).items():
 assert hashlib.sha256((v/path).read_bytes()).hexdigest()==expected,path
icon=(v/'assets/command-icon.png').read_bytes();assert len(icon)==169316 and hashlib.sha256(icon).hexdigest()=='6ae8ae29d853b86b840fe54297a2a9d66520de7c7ce86cc206083d6ddfc3add9'
assert icon==(v/'fixtures/original/extensions/file-tree-generator/assets/command-icon.png').read_bytes()
if (v.parent/'.git').exists():
 source='2d1f4b0bcbf1cfd97e0fe3aa1bdfd0feba18c3a0'
 for path in subprocess.check_output(['git','-C',str(v.parent),'ls-tree','-r','--name-only',source]).decode().splitlines():
  original=subprocess.check_output(['git','-C',str(v.parent),'show',source+':'+path]);assert original==(v/'fixtures/own'/path).read_bytes(),path
  if path!='README.md':assert original==(v.parent/path).read_bytes(),path
 # Root README may contain only a trailing discovery link.
 original=subprocess.check_output(['git','-C',str(v.parent),'show',source+':README.md']);assert (v.parent/'README.md').read_bytes()==original+b'\n## Rust Native V2\n\nSee [v2/README.md](v2/README.md) for the standalone Rust port and integration instructions.\n'
 for path in json.loads((v/'fixtures/hashes.json').read_text()):
  check=subprocess.run(['git','-C',str(v.parent),'check-ignore','--no-index',str(v/path)],capture_output=True)
  assert check.returncode==1, 'Required fixture ignored: '+path
print(f'PASS full pinned schema + {len(negative)} negative cases; fixture hashes, exact icon, V1 byte preservation and Git visibility')
