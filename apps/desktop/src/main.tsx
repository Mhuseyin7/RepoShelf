import React, { useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import "./styles.css";

type Generated = { name:string; bytes:number };
type Repo = { name:string; path:string; branch:string|null; detached_head:boolean; dirty:boolean; changes:{modified:number;untracked:number}; ahead:number|null; behind:number|null; remotes:string[]; last_commit_at:string|null; disk:{total_bytes:number; generated:Generated[]}; technologies:string[] };
const bytes=(n:number)=> n<1024?`${n} B`:n<1048576?`${(n/1024).toFixed(1)} KB`:n<1073741824?`${(n/1048576).toFixed(1)} MB`:`${(n/1073741824).toFixed(1)} GB`;
function App(){
 const [repos,setRepos]=useState<Repo[]>([]), [root,setRoot]=useState(""), [query,setQuery]=useState(""), [busy,setBusy]=useState(false), [error,setError]=useState<string|null>(null);
 const visible=useMemo(()=>{const q=query.toLowerCase();return repos.filter(r=>[r.name,r.path,r.branch,...r.remotes,...r.technologies].filter(Boolean).join(" ").toLowerCase().includes(q))},[repos,query]);
 const scan=async()=>{ if(!root.trim()) return setError("Enter a folder to scan."); setBusy(true); setError(null); try { setRepos(await invoke<Repo[]>("scan",{roots:[root]})); } catch(e){setError(String(e))} finally{setBusy(false)} };
 const open=async(path:string)=>{try{await invoke("open_folder",{path})}catch(e){setError(String(e))}};
 const totals={dirty:repos.filter(r=>r.dirty).length,ahead:repos.filter(r=>(r.ahead??0)>0).length,behind:repos.filter(r=>(r.behind??0)>0).length,size:repos.reduce((a,r)=>a+r.disk.total_bytes,0)};
 return <main><header><div><p className="eyebrow">LOCAL-FIRST WORKSPACE INVENTORY</p><h1>RepoShelf</h1></div><div className="scan"><input value={root} onChange={e=>setRoot(e.target.value)} placeholder="C:\Projects or ~/Code"/><button onClick={scan} disabled={busy}>{busy?"Scanning…":"Scan folders"}</button></div></header>
 <section className="metrics"><Metric label="Repositories" value={repos.length}/><Metric label="Dirty" value={totals.dirty}/><Metric label="Ahead" value={totals.ahead}/><Metric label="Behind" value={totals.behind}/><Metric label="Disk usage" value={bytes(totals.size)}/></section>
 <section className="toolbar"><input value={query} onChange={e=>setQuery(e.target.value)} placeholder="Search name, path, branch, remote, technology"/><span>{visible.length} shown</span></section>{error&&<p className="error">{error}</p>}
 <section className="list">{visible.map(r=><article key={r.path}><div className="title"><h2>{r.name}</h2><p>{r.path}</p></div><div className="status"><span className={r.dirty?"pill dirty":"pill"}>{r.dirty?`${r.changes.modified} modified · ${r.changes.untracked} untracked`:"Clean"}</span><span>{r.branch??(r.detached_head?"Detached HEAD":"Bare")}</span><span>↑{r.ahead??0} ↓{r.behind??0}</span></div><div className="details"><span>{r.technologies.join(" · ")||"Unknown stack"}</span><span>{bytes(r.disk.total_bytes)}</span>{r.disk.generated.map(d=><span key={d.name}>{d.name}: {bytes(d.bytes)}</span>)}</div><button className="open" onClick={()=>open(r.path)}>Open folder</button></article>)}{!busy&&visible.length===0&&<div className="empty">Choose a scan root to discover repositories. RepoShelf never changes a repository during scanning.</div>}</section></main> }
function Metric({label,value}:{label:string;value:string|number}){return <div className="metric"><span>{label}</span><strong>{value}</strong></div>}
createRoot(document.getElementById("root")!).render(<React.StrictMode><App/></React.StrictMode>);
