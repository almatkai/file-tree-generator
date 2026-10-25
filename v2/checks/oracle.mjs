// Executes the own pinned TS, never a reimplementation of the expected algorithm.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import fs from 'node:fs';
import vm from 'node:vm';
import { spawnSync } from 'node:child_process';
const v=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const require=createRequire(path.join(v,'fixtures/own/package.json'));
const esbuild=require('esbuild');
for (const pkg of ['lodash','tsx','esbuild','react']) {
 const locked=path.join(v,'fixtures/locked',pkg,'package.json');
 assert.equal(fs.readFileSync(locked,'utf8'),fs.readFileSync(path.join(v,'fixtures/own/node_modules',pkg,'package.json'),'utf8'));
}
async function load(entry, stub=false) {
 const result=await esbuild.build({stdin:{contents:entry,resolveDir:path.join(v,'fixtures/own'),loader:'tsx'},bundle:true,write:false,platform:'node',format:'cjs',jsx:'automatic',plugins:stub?[{name:'OS-React-RayCast-boundaries',setup(b){b.onResolve({filter:/^(react(?:\/jsx-runtime)?|@raycast\/api)$/},a=>({path:a.path,namespace:'boundary'}));b.onLoad({filter:/.*/,namespace:'boundary'},a=>({contents:a.path==='react'?'export const useState=globalThis.boundary.useState;':a.path==='react/jsx-runtime'?'export const jsx=(type,props)=>({type,props}); export const jsxs=jsx;':'const component=(name)=>Object.assign(name,{TextArea:"TextArea",Dropdown:Object.assign("Dropdown",{Item:"Item"}),Checkbox:"Checkbox"}); export const Form=component("Form"); export const ActionPanel="ActionPanel"; export const Action={CopyToClipboard:"CopyToClipboard"};'}));}}]:[]});
 const module={exports:{}};let states=[],cursor=0;
 const boundary={useState:(initial)=>{const i=cursor++;if(!(i in states))states[i]=initial;return [states[i],value=>{states[i]=value}];}};
 const context={module,exports:module.exports,require,globalThis:{boundary}};
 vm.runInNewContext(result.outputFiles[0].text,context);
 return {exports:module.exports,render:()=>{cursor=0;return module.exports.default();}};
}
const {exports:ts}=await load('export {parseInput} from "./src/parse-input"; export {generateTree,defaultOptions} from "./src/generate-tree";');
const cases=['','a','a\n b\n  c\nd','a\r\nb\rc\n\n','a\n\ta\n  b','- a\n  - b','-a\n-\t b\n-  c','    \n\t\r\n','a/ \n b','a\na\n a\n a','😀\n 中\n  Қазақстан','\ufeff- a\n\u00a0b','a\u2028b\u2029c','\u0085x\n\u200bx','a\n  \n    b','a\\b\na/b\n../x','<script>\n `quote`\n  [markdown]','a\n  - \n    x',' a\nb\n    c\n d','a\0b\n c','a\n\v- b\n\f c'];
const spaces=['\t',' ','  ','\u00a0','\ufeff','\u2028','\u2029','\u1680','\u2000','\u200a','\u202f','\u205f','\u3000','\u0085','\u180e','\u200b'];
for (const w of spaces) cases.push(`${w}- x\r\n${w}${w}y/ ${w}\n${w}z`);
let seed=0x432123; const rand=n=>{seed=(Math.imul(seed,1664525)+1013904223)>>>0;return seed%n};
const names=['a','b/','name/ ','😀','日本','Қазақ','- x','-\tq','-  x','..','C:\\a','<>&"','a\u2028b','a\0b'];
for(let i=0;i<500;i++){let s='';for(let j=0,n=1+rand(25);j<n;j++)s+=spaces[rand(spaces.length)].repeat(rand(6))+names[rand(names.length)]+['\n','\r','\r\n'][rand(3)];cases.push(s);}
const queries=[];
for(const input of cases)for(const charset of ['ascii','utf-8'])for(const trailingDirSlash of [false,true])for(const fullPath of [false,true])for(const rootDot of [false,true])queries.push({input,options:{charset,trailingDirSlash,fullPath,rootDot}});
const binary=process.env.FILE_TREE_ENGINE || path.join(v,'target/debug/examples/oracle_bridge');
const run=spawnSync(binary,[],{input:queries.map(q=>JSON.stringify(q)).join('\n')+'\n',encoding:'utf8',maxBuffer:100*1024*1024});assert.equal(run.status,0,run.stderr);
const outputs=run.stdout.trimEnd().split('\n').map(s=>JSON.parse(s));assert.equal(outputs.length,queries.length);
for(let i=0;i<queries.length;i++){const q=queries[i];assert.equal(outputs[i],ts.generateTree(ts.parseInput(q.input),q.options),JSON.stringify(q));}
// Execute actual own UI with only React state/JSX and Raycast descriptors stubbed.
const ui=await load('export {default} from "./src/index";',true);let tree=ui.render();const fields=()=>tree.props.children;assert.deepEqual(Array.from(fields(),f=>f.props.id),['input','charset','trailingDirSlash','fullPath','rootDot','output']);
assert.equal(fields()[1].props.storeValue,true);assert.equal(fields()[2].props.storeValue,undefined);assert.equal(fields()[3].props.storeValue,true);assert.equal(fields()[4].props.storeValue,true);
fields()[0].props.onChange('a\n b');tree=ui.render();assert.equal(fields()[5].props.value,'a/\n└── b');
fields()[1].props.onChange('ascii');tree=ui.render();assert.equal(fields()[5].props.value,'a/\n`-- b');
assert.equal(tree.props.actions.props.children.props.content,fields()[5].props.value);
fields()[5].props.onChange('forged');tree=ui.render();assert.equal(fields()[5].props.value,'a/\n`-- b');
console.log(`PASS ${queries.length} TS→Rust parity cases; actual TSX state/actions/storeValue checked; no runtime lodash calls (own parser/renderer import no libraries)`);
