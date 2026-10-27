// Compile and execute real pinned beta parsers/reconciliation/surface routing.
import assertBase from 'node:assert/strict';
let checks=0; const assert=new Proxy(assertBase,{get(target,name){const value=target[name];return typeof value==='function'?(...args)=>{checks++;return value(...args)}:value;}});
import {createRequire} from 'node:module';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import fs from 'node:fs';
import vm from 'node:vm';
const v=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const require=createRequire(path.join(v,'fixtures/own/package.json'));const esbuild=require('esbuild');
const dir=path.join(v,'fixtures/beta/src/renderer/src/raycast-api');
const actual=new Set(['form-runtime','detail-runtime','list-runtime','index','action-runtime','action-runtime-overlay','action-runtime-registry']);
const react=`export const useMemo=f=>f(); export const useCallback=f=>f; export const useEffect=()=>{}; export const useRef=v=>({current:v}); export const useState=v=>[typeof v==='function'?v():v,()=>{}]; export const forwardRef=f=>f; export const useImperativeHandle=()=>{}; export const createContext=v=>({Provider:'Provider',_v:v}); export const useContext=c=>c._v; export default {};`;
const stub=`export const Hint=()=>null, HintBar=Hint, Kbd=Hint, ViewHeader=Hint, Markdown=Hint, GridRuntime=Hint; export const cx=(...a)=>a.join(' '); export const buildColorFormatRows=()=>[],buildColorConversionResults=()=>[]; export const NavigationContext={Provider:'Nav'};export const Clipboard={}, environment={}; export const getPreferenceValues=()=>({}),open=()=>{},setRuntimeContext=()=>{},showHUD=()=>{},showToast=()=>{},useCachedPromise=()=>{},useFetch=()=>{},useNavigation=()=>{},usePromise=()=>{};`;
const result=await esbuild.build({stdin:{contents:`export * from './form-runtime';export * from './list-runtime';export * from './detail-runtime';export * from './index';`,resolveDir:dir,loader:'tsx'},bundle:true,write:false,platform:'node',format:'cjs',jsx:'automatic',plugins:[{name:'headless-boundaries',setup(b){
 b.onResolve({filter:/.*/},a=>{
  if(a.path==='react'||a.path==='react/jsx-runtime')return {path:a.path,namespace:'boundary'};
  if(a.path.startsWith('./')&&actual.has(a.path.slice(2)))return {path:path.join(dir,a.path+'.tsx')};
  return {path:a.path,namespace:'boundary'};
 });
 b.onLoad({filter:/.*/,namespace:'boundary'},a=>({contents:a.path==='react'?react:a.path==='react/jsx-runtime'?'export const jsx=(type,props)=>({type,props});export const jsxs=jsx;export const Fragment="Fragment";':stub}));
 b.onLoad({filter:/\.tsx$/},a=>{let s=fs.readFileSync(a.path,'utf8');if(a.path.endsWith('/form-runtime.tsx'))s+='\nexport {collectFormItems};';if(a.path.endsWith('/list-runtime.tsx'))s+='\nexport {parseListRows};';if(a.path.endsWith('/detail-runtime.tsx'))s+='\nexport {markdownFromNode};';return {contents:s,loader:'tsx',resolveDir:dir};});
}}]});
const module={exports:{}};vm.runInNewContext(result.outputFiles[0].text,{module,exports:module.exports,require});const api=module.exports;
const wire=JSON.parse(fs.readFileSync(process.env.FILE_TREE_WIRE_OUT||path.join(v,'checks/materialized-next/wire.json'),'utf8'));
const fields=root=>api.collectFormItems(root).filter(i=>i.type==='field').map(i=>i.field);
const before=fields(wire.before.root),after=fields(wire.after.root);assert.equal(before.length,6);assert.equal(after.length,6);assert.equal(after.find(f=>f.id==='output').value,'one/\n└── two');
assert.equal(before.find(f=>f.id==='charset').options.length,2);
const pending={...api.reconcileFormValues(before),input:'one\n two',charset:'ascii',fullPath:true};
const reconciled=api.reconcileFormValues(after,pending,before);assert.equal(reconciled.charset,'ascii');assert.equal(reconciled.fullPath,true);assert.equal(reconciled.input,'one\n two');assert.equal(reconciled.output,'one/\n└── two');
assert.equal(api.parseListRows(wire.before.root).length,0);assert.equal(api.markdownFromNode(wire.before.root),'');
const parsed=api.parseListRows({type:'List',children:[{type:'List.Section',props:{title:'S'},children:[{type:'List.Item',props:{id:'id',title:'literal',actionIds:['cap',42]}}]}]});assert.equal(parsed[0].id,'id');assert.equal(parsed[0].actionIds.length,1);
let calls=[],backs=0;
const surface=api.ExtensionRuntimeSurface({...wire.after,onInvokeAction:(...args)=>calls.push(args),onBack:()=>backs++,onSearchTextChanged:()=>{},onLoadMore:()=>{}});
function find(node,predicate){if(!node||typeof node!=='object')return null;if(predicate(node))return node;const c=node.props?.children;for(const child of Array.isArray(c)?c:[c]){const found=find(child,predicate);if(found)return found;}return null;}
const form=find(surface,node=>node.props?.onChangeField&&node.props?.onSubmitForm);assert.ok(form);
const capability=after.find(f=>f.id==='input').actionId;form.props.onChangeField(capability,'new');assert.equal(calls[0][0],capability);assert.equal(JSON.stringify(calls[0][1]),JSON.stringify({value:'new'}));
form.props.onSubmitForm(reconciled);assert.equal(calls[1][0],wire.after.actions[0].id);assert.equal(calls[1][1],reconciled);
form.props.onBack();assert.equal(backs,1);
console.log(`PASS ${checks} actual beta Form/List/Detail parser, queued draft, submit/change/back routing assertions (React/DOM/OS boundaries stubbed)`);
