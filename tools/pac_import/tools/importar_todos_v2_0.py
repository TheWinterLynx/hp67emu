#!/usr/bin/env python3
"""Local, offline HP67PACS preparation and transactional catalog installation."""
import argparse
from collections import defaultdict
import hashlib
import html
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys

import fitz
import numpy as np
from PIL import Image, ImageDraw
import convert_hp67_card as media
import artwork_v2_1 as art
from normalize_pack_names_v2_2 import NAMES, normalize

HERE=Path(__file__).parent
CONFIG={
 'standard':('HP-67 Standard Pac','HP-67 Standard Pac'),
 'games':('HP-67 Games Pac 1','HP-67 Games Pac 1'),
 'business':('Business Decisions Pac','Business Decisions Pac'),
 'civil':('HP-67CE','HP-67 Civil Engineering Pac I'),
 'ee':('HP-67 EE Pac I','HP-67 EE Pac I'),
 'math':('HP-67 Math Pac I','HP-67 Math Pac I'),
 'me':('HP-67 ME Pac','HP-67 ME Pac'),
 'stat':('HP-67 Stat Pac I','HP-67 Stat Pac I'),
 'clinical':('HP67-Clinical-Lab-and-Nuclear-Medicine-Native-v1.0','HP-67 Clinical Lab and Nuclear Medicine Pac'),
 'navigation':('HP67-Navigation-Pac-I-Native-v1.0','HP-67 Navigation Pac I'),
 'surveying':('HP67-Surveying-Pac-I-Native-v1.0','HP-67 Surveying Pac I'),
 'eaton':('HP67de','Dave Eaton HP-67'),
 'eaton2026':('Programs67_97','Dave Eaton HP-67/97 2026'),
}
CONFIG={key:(folder,NAMES.get(name,name)) for key,(folder,name) in CONFIG.items()}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def writejson(p,d):p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(d,indent=2,ensure_ascii=False),encoding='utf-8')
def nat(s):return [int(x) if x.isdigit() else x.lower() for x in re.split(r'(\d+)',s)]
def slug(s):return re.sub(r'[^A-Za-z0-9._-]+','-',s).strip('-')
def quoted(s):return json.dumps(s,ensure_ascii=False)
def need(ok,msg):
 if not ok:raise ValueError(msg)
def copy_checked(source,target,expected=None):
 expected=expected or sha(source)
 with source.open('rb') as src,target.open('wb') as dst:
  shutil.copyfileobj(src,dst,1024*1024)
 need(sha(target)==expected,f'Incomplete/changed file copy: {source} -> {target}')
def validate_card(p):
 tracks=media.parse_native(p)
 need(any(t.recorded for t in tracks),f'Empty card: {p}')
 for t in tracks:
  if t.recorded:need(sum(t.words[:33]) & media.WORD_MASK == t.words[33],f'Checksum: {p}')
 return sum(t.recorded for t in tracks)

def group_hpp(folder,key):
 groups=defaultdict(list)
 for p in sorted(folder.rglob('*.hpp'),key=lambda x:nat(x.as_posix())):
  t=media.parse_hpp(p)
  need(sum(t.words[:33]) & media.WORD_MASK == t.words[33],f'Bad source checksum: {p}')
  if key=='math' or key=='stat':
   m=re.search(r'L(\d\d)(A\d)?',t.bitmap_name,re.I);need(m,f'Unrecognized bitmap id: {p}')
   ref=('MA1-' if key=='math' else 'ST1-')+m[1]+(m[2] or 'A')
  elif key=='eaton':
   ref='DE-'+p.name[0]+('-'+p.name.split()[1] if p.name.startswith('F ') else '')
  elif key=='eaton2026':
   m=re.match(r'([AE])(\d\d)',p.name);need(m,f'Unknown Eaton file: {p}')
   # Continuation pairs follow the explicitly numbered, adjacent source stream.
   block=m[1];n=int(m[2]);ref=f'DE26-{block}{n:02}'
   if t.header_id==4:
    prev=[r for r,v in groups.items() if r.startswith('DE26-'+block) and int(r[-2:])==n-1]
    need(len(prev)==1,f'Orphan continuation: {p}');ref=prev[0]
  else:
   m=re.match(r'([A-Z]+\d?-\d\d[A-Z]?\d?)',p.name);need(m,f'Unknown PAC filename: {p}')
   ref=m[1]
  groups[ref].append((p,t))
 need(groups,f'No HPP media in {folder}')
 result=[]
 for ref,parts in sorted(groups.items(),key=lambda x:nat(x[0])):
  first=second=None
  for p,t in parts:
   side=t.inferred_track
   physical=re.search(r'_([12])(?:\s|\.)',p.name)
   if physical:
    # Official source filenames label the physical insertion end. Header 3/4
    # labels payload meaning, not card end (SI and ME heat exchangers differ).
    side=int(physical[1])
   if side==1:need(first is None,f'Duplicate first track: {ref}');first=t.words
   else:need(second is None,f'Duplicate second track: {ref}');second=t.words
  if key not in ('eaton','eaton2026'):
   app=int(re.search(r'-(\d\d)',ref)[1])
  else:app=None
  title=parts[0][0].stem
  title=re.sub(r'^[A-Z]+\d?-\d\d[A-Z]?\d?_\d\s*','',title)
  title=re.sub(r'^67-(?:Math|Stat)\s+L\S+\s*','',title) or parts[0][1].card_name
  if key in ('math','stat'):title=parts[0][1].card_name
  result.append({'reference':ref,'app':app,'title':title,'kind':'data' if all(t.header_id in (1,2) for _,t in parts) else 'program','bytes':media.native_bytes(first,second),'sources':[p for p,_ in parts]})
 return result

def native_entries(folder,key):
 mpath=folder/'manifest.json';need(mpath.is_file(),f'Missing {mpath}')
 entries=[]
 for e in json.loads(mpath.read_text(encoding='utf-8')):
  p=folder/e['file'];need(p.is_file() and sha(p)==e['native_sha256'],f'Changed native card: {p}')
  validate_card(p)
  appmatch=re.search(r'/(?:CM|NV1|SU1)-(\d\d)/',e['source_path'])
  app=int(appmatch[1]) if appmatch else (17 if 'LambertData' in e['source_path'] else 18 if 'MercatorData' in e['source_path'] else None)
  need(app is not None,f'Cannot bind native source: {p}')
  ref=('CL1' if key=='clinical' else 'NAV' if key=='navigation' else 'SU1')+f'-{app:02}'+('-D-' if e['kind']=='data' else '-')+hashlib.sha256(e['source_path'].encode()).hexdigest()[:8]
  entries.append({'reference':ref,'app':app,'title':e['title'],'kind':e['kind'],'bytes':p.read_bytes(),'sources':[p,mpath],'original':e})
 return entries

def face_for_native(e,key,faces):
 original=e['original'];path=original['source_path'];title=original['title']
 if key=='clinical':return faces[0]
 if key=='navigation':
  if e['app']==5:return faces[1 if e['kind']=='data' else 0]
  if e['app']==8:
   m=re.match(r'08([a-f])',title)
   return faces[ord(m[1])-ord('a')+1] if m else faces[0]
  return faces[0]
 if e['app'] in (17,18):
  if e['kind']=='data':return faces[2]
  return faces[1 if re.search(r'/SU1-\d\db\.',path) else 0]
 if e['app']==19:
  b=bool(re.search(r'/SU1-19b',path));return faces[(2 if b else 0)+(1 if e['kind']=='data' else 0)]
 return faces[0]

def extract_faces(pdf,profile,work,key):
 folder=work/'payload/assets/hp67-imported-artwork';folder.mkdir(parents=True,exist_ok=True)
 original=work/'review'/key;original.mkdir(parents=True,exist_ok=True)
 d=fitz.open(pdf);faces=defaultdict(list)
 for i,row in enumerate(profile['rows']):
  page=d[row['page']-1];box=fitz.Rect(row['rect'])
  box.y1=max(box.y1,box.y0+box.width/(art.CARD_WIDTH_MM/art.CARD_HEIGHT_MM))
  box=fitz.Rect(box.x0-2,box.y0-2,box.x1+2,box.y1+2)&page.rect
  pix=page.get_pixmap(matrix=fitz.Matrix(600/72,600/72),clip=box,alpha=False)
  image=Image.frombytes('RGB',(pix.width,pix.height),pix.samples)
  a=np.asarray(image);black=a.min(axis=2)<65
  yy=np.flatnonzero(black.sum(axis=1)>image.width*.35);need(len(yy)>100,f'Empty card crop {key} {i}')
  xx=np.flatnonzero(black[yy].sum(axis=0)>len(yy)*.18);need(len(xx)>100,f'Empty card width {key} {i}')
  image=image.crop((int(xx.min()),int(yy.min()),int(xx.max())+1,int(yy.max())+1))
  # Both variants are kept: source pixels for inspection, canonical face for UI.
  image.save(original/(row['id']+'-source.png'),dpi=(600,600))
  # Some PDF scans print the card a few percent smaller. Calibrate its width
  # uniformly before the physical canvas/mask: never stretch X and Y separately.
  width=art.card_pixel_size(600)[0]
  height=art.card_pixel_size(600)[1]
  scale=min(width/image.width,height/image.height)
  calibrated=image.resize((min(width,round(image.width*scale)),min(height,round(image.height*scale))),Image.Resampling.LANCZOS)
  calibrated.putalpha(art.card_silhouette_mask((width,height),600).resize(calibrated.size,Image.Resampling.NEAREST))
  canonical=art.normalize_card_artwork(calibrated,600)
  need(canonical.size==art.card_pixel_size(600),'Wrong canonical dimensions')
  need(canonical.getchannel('A').getextrema()==(255,255),'Transparent substrate')
  target=folder/(row['id']+'.png');canonical.save(target,dpi=(600,600))
  faces[row['app']].append({'id':row['id'],'path':target.relative_to(work/'payload').as_posix(),'page':row['page'],'rect':row['rect']})
 sheet=Image.new('RGB',(1340,((len(profile['rows'])+1)//2)*135),'#cccccc');draw=ImageDraw.Draw(sheet)
 for i,row in enumerate(profile['rows']):
  im=Image.open(folder/(row['id']+'.png')).convert('RGB');im.thumbnail((650,105));x=(i%2)*670;y=(i//2)*135
  draw.text((x+8,y+3),f'{row["id"]} | PDF p{row["page"]} | app {row["app"]}',fill='black');sheet.paste(im,(x+8,y+25))
 sheet.save(original/'sheet.jpg',quality=95)
 return faces

def prepare(args):
 source=args.source_root.resolve();work=args.work.resolve();need(source.is_dir(),f'Missing {source}')
 need(work!=source and source not in work.parents,'Work must be outside HP67PACS')
 need(not work.exists(),f'Work already exists: {work}; choose a new empty output')
 profiles=json.loads((HERE/'profiles-v2.0.json').read_text())
 # Resolve and validate every prerequisite before creating output or altering repo.
 for key,(folder,_) in CONFIG.items():need((source/folder).is_dir(),f'Missing folder: {source/folder}')
 for key,p in profiles.items():
  pdf=source/p['file'];need(pdf.parent==source and pdf.is_file(),f'Missing ROOT PDF: {pdf}')
  need(sha(pdf)==p['sha256'],f'PDF differs from reviewed edition: {pdf}. No nested PDF fallback is permitted.')
 entries=[];sources={};items=[]
 for key,(folder,pack) in CONFIG.items():
  found=native_entries(source/folder,key) if key in ('clinical','navigation','surveying') else group_hpp(source/folder,key)
  for e in found:
   for p in e['sources']:sources[str(p)]=sha(p)
   e['pack']=pack;e['key']=key
  items.append((key,found))
 print('Prerequisites OK: 13 source folders, 11 root PDFs; preparing offline.',flush=True)
 work.mkdir(parents=True)
 for index,(key,found) in enumerate(items,1):
  faces={}
  if key in profiles:
   p=profiles[key];pdf=source/p['file'];sources[str(pdf)]=p['sha256']
   faces=extract_faces(pdf,p,work,key)
   target=work/'payload/programs/HP67/ImportedPACs/manuals'/p['file'];target.parent.mkdir(parents=True,exist_ok=True);copy_checked(pdf,target,p['sha256'])
  byapp=defaultdict(list)
  for e in found:byapp[e['app']].append(e)
  for app,group in byapp.items():
   for j,e in enumerate(group):
    face=None
    if key in ('clinical','navigation','surveying'):face=face_for_native(e,key,faces[app])
    elif key in profiles:
     if key=='standard' and app==15:pass  # Reviewed root scan depicts SD-15C, not A.
     else:
      need(len(group)==len(faces[app]),f'Ambiguous image mapping: {key}/{app}')
      face=faces[app][j]
    ref=e['reference'];rel=f'programs/HP67/ImportedPACs/{key}/{slug(ref)}.hp67card';target=work/'payload'/rel;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(e['bytes'])
    tracks=validate_card(target)
    out={k:v for k,v in e.items() if k not in ('bytes','sources')}
    out.update({'file':rel,'tracks':tracks,'sha256':sha(target),'face':face,'inputs':[{'path':p.relative_to(source).as_posix(),'sha256':sha(p)} for p in e['sources']],'pdf':f'programs/HP67/ImportedPACs/manuals/{profiles[key]["file"]}' if face else None})
    entries.append(out)
  print(f'[{index}/13] {key}: {len(found)} native cards'+(f', {sum(map(len,faces.values()))} source faces' if faces else ', no PAC PDF/artwork supplied'),flush=True)
 # Native C fixture already belongs to the emulator; attach the correct printed face.
 p=profiles['standard'];cface={'id':p['rows'][-1]['id'],'path':f'assets/hp67-imported-artwork/{p["rows"][-1]["id"]}.png','page':p['rows'][-1]['page'],'rect':p['rows'][-1]['rect']}
 refs=[e['reference'] for e in entries];need(len(set(refs))==len(refs),'Duplicate catalog references')
 writejson(work/'payload/programs/HP67/ImportedPACs/import_manifest.json',{'version':'2.0','entries':entries,'pdfs':{k:{'file':p['file'],'sha256':p['sha256']} for k,p in profiles.items()},'source_faces':sum(len(p['rows']) for p in profiles.values()),'notes':['Root PDFs only; printed revisions may differ from filename revisions.','Standard PDF diagnostic face is SD-15C and must not label SD1-15A.','Generic Lambert/Mercator faces are shared by zone datasets.','NAV source 12 is Horizon, printed NAV-13A; source 13 is Radar, printed NAV-12A.']})
 hashes={p.relative_to(work/'payload').as_posix():sha(p) for p in (work/'payload').rglob('*') if p.is_file()}
 plan={'version':'2.0','source_root':str(source),'sources':sources,'payload_hashes':hashes,'entries':entries,'sd15c_face':cface,'standard_pdf':'programs/HP67/ImportedPACs/manuals/'+p['file'],'source_faces':sum(len(p['rows']) for p in profiles.values())}
 writejson(work/'plan.json',plan)
 rows=[]
 for key in profiles:rows.append(f'<h2>{html.escape(key)}</h2><img style="max-width:100%" src="{key}/sheet.jpg">')
 (work/'review/index.html').write_text('<!doctype html><meta charset="utf-8"><title>HP67 PAC Review v2.0</title><h1>Root PDF card extraction</h1><p>600 DPI. Source crops are preserved beside the canonical previews. Program revisions remain recorded separately from printed face revisions.</p>'+''.join(rows),encoding='utf-8')
 print(f'PREPARED: {len(entries)} cards; {plan["source_faces"]} extracted source faces. Review: {work/"review/index.html"}',flush=True)

def rust_entry(e):
 pdf='Some('+quoted(e['pdf'])+')' if e['pdf'] else 'None'
 face='Some('+quoted(e['face']['path'])+')' if e['face'] else 'None'
 return f'''    ProgramLibraryEntry {{
        pack: {quoted(e['pack'])},
        reference: {quoted(e['reference'])},
        title: {quoted(e['title'])},
        source_pdf: {pdf},
        artwork_path: {face},
        parts: &[hpp!({quoted('/'+e['file'])})],
        artwork: catalog_artwork({quoted(e['title'])}, {quoted(e['reference'])}),
    }},'''

def catalog_blocks(text):
 marker='pub const PROGRAM_LIBRARY: &[ProgramLibraryEntry] = &['
 need(text.count(marker)==1,'Unsupported catalog declaration')
 start=text.index(marker)+len(marker);end=text.index('\n];',start)
 blocks=re.findall(r'^    ProgramLibraryEntry \{.*?^    \},',text[start:end],re.M|re.S)
 need(blocks and not re.sub(r'^    ProgramLibraryEntry \{.*?^    \},','',text[start:end],flags=re.M|re.S).strip(),'Unsupported catalog syntax')
 return start,end,blocks

def run(repo,*args):
 r=subprocess.run(['git','-C',str(repo),*args],capture_output=True,text=True)
 need(r.returncode==0,r.stderr or 'git failed');return r.stdout.strip()

def install(args):
 repo=args.repo.resolve();work=args.work.resolve();plan=json.loads((work/'plan.json').read_text(encoding='utf-8'))
 need((repo/'Cargo.toml').is_file(),'Not hp67emu repository')
 need(not run(repo,'status','--porcelain','--untracked-files=no'),'Tracked changes present; commit or stash before install')
 need(run(repo,'branch','--show-current')=='main','Switch to main before install')
 for path,digest in plan['sources'].items():need(sha(Path(path))==digest,f'Source changed since prepare: {path}')
 for path,digest in plan['payload_hashes'].items():need(sha(work/'payload'/path)==digest,f'Prepared output changed: {path}')
 extra={p.relative_to(work/'payload').as_posix() for p in (work/'payload').rglob('*') if p.is_file()}-set(plan['payload_hashes'])-{'assets/hp67-card-artwork-atlas.png'}
 need(not extra,f'Unexpected prepared files: {sorted(extra)}')
 lib=repo/'src/program_library.rs';app=repo/'src/app.rs';text=lib.read_text(encoding='utf-8');apptext=app.read_text(encoding='utf-8')
 start,end,blocks=catalog_blocks(text)
 existing={re.search(r'reference: "([^"]+)"',b)[1]:b for b in blocks}
 refs=list(existing);entries={e['reference']:e for e in plan['entries']}
 for ref,e in entries.items():
  if ref not in existing:refs.append(ref)
  existing[ref]=rust_entry(e)
 need('SD-15C' in existing,'This repository lacks the exact SD-15C native fixture; update main first')
 block=existing['SD-15C']
 block=re.sub(r'source_pdf: (?:None|Some\("[^"]*"\))',f'source_pdf: Some({quoted(plan["standard_pdf"])})',block)
 block=re.sub(r'artwork_path: (?:None|Some\("[^"]*"\))',f'artwork_path: Some({quoted(plan["sd15c_face"]["path"])})',block)
 existing['SD-15C']=block
 # Build all rendered rows in catalog order. Repeated generic data faces get
 # independent atlas rows; original PNGs are stored only once.
 facepaths={};tracks=0
 for ref in refs:
  block=existing[ref];face=None
  if ref in entries:face=entries[ref]['face']
  elif ref=='SD-15C':face=plan['sd15c_face']
  else:need('source_pdf: None' in block,f'Unhandled existing PDF-backed entry: {ref}')
  if face:
   facepaths[ref]=work/'payload'/face['path']
  if ref in entries:tracks+=entries[ref]['tracks']
  else:
   paths=re.findall(r'"(/programs/[^"\n]+\.(?:hpp|hp67card))"',block)
   need(paths,f'Cannot count legacy media: {ref}')
   for path in paths:
    p=repo/path.lstrip('/')
    tracks+=validate_card(p) if p.suffix=='.hp67card' else 1
 # Preserve the original atlas geometry/order and its SD-01A mark regression.
 oldrows=sorted(re.findall(r'"([^"]+)" => Some\((\d+)\)',text),key=lambda x:int(x[1]))
 ordered=['SD-15C' if ref=='SD1-15A' else ref for ref,_ in oldrows]
 ordered=[ref for ref in ordered if ref in facepaths]
 ordered.extend(ref for ref in refs if ref in facepaths and ref not in ordered)
 need(len(ordered)==len(facepaths) and len(set(ordered))==len(ordered),'Atlas row order')
 rowmap=list(zip(ordered,range(len(ordered))))
 atlasimages=[Image.open(facepaths[ref]).convert('RGBA').resize((499,80),Image.Resampling.BOX) for ref in ordered]
 atlas=Image.new('RGBA',(499,80*len(atlasimages)),(0,0,0,255))
 for i,im in enumerate(atlasimages):atlas.paste(im,(0,i*80))
 atlaspath=work/'payload/assets/hp67-card-artwork-atlas.png';atlas.save(atlaspath)
 new=normalize(text[:start]+'\n'+'\n'.join(existing[r] for r in refs)+text[end:])
 function='    pub fn artwork_atlas_row(&self) -> Option<usize> {\n        match self.reference {\n'+''.join(f'            {quoted(r)} => Some({n}),\n' for r,n in rowmap)+'            _ => None,\n        }\n    }'
 new,n=re.subn(r'    pub fn artwork_atlas_row\(&self\) -> Option<usize> \{.*?\n    \}',lambda m:function,new,count=1,flags=re.S);need(n==1,'Missing atlas mapping function')
 new,n=re.subn(r'assert_eq!\(rows.len\(\), \d+\);',f'assert_eq!(rows.len(), {len(rowmap)});',new);need(n==1,'Missing artwork count test')
 new,n=re.subn(r'\(0\.\.\d+\)\.collect::<Vec<_>>\(\)',f'(0..{len(rowmap)}).collect::<Vec<_>>()',new);need(n==1,'Missing atlas continuity test')
 new,n=re.subn(r'assert_eq!\(PROGRAM_LIBRARY.len\(\), \d+\);',f'assert_eq!(PROGRAM_LIBRARY.len(), {len(refs)});',new);need(n==1,'Missing catalog count test')
 check_start=new.index('    fn catalog_contains_all_built_in_program_media()')
 check_end=new.index('\n    }',check_start)
 check,n=re.subn(r'(\.sum::<usize>\(\),\s*)\d+(\s*\n\s*\);)',lambda m:m[1]+str(tracks)+m[2],new[check_start:check_end]);need(n==1,'Missing track count test')
 new=new[:check_start]+check+new[check_end:]
 appnew,n=re.subn(r'const CARD_ARTWORK_ATLAS_ROWS: u32 = \d+;',f'const CARD_ARTWORK_ATLAS_ROWS: u32 = {len(rowmap)};',apptext);need(n==1,'Missing app atlas rows')
 # All preparation and Rust edits above happen before any repository mutation.
 files={Path('src/program_library.rs'):new.encode(),Path('src/app.rs'):appnew.encode()}
 for companion in ['docs/files/src/program_library.rs.md','docs/files/src/app.rs.md']:
  p=repo/companion;need(p.is_file(),f'Missing companion {p}')
  files[Path(companion)]=(p.read_text(encoding='utf-8')+'\n\nPAC import v2.0: all imported media use native HP67CARD; reviewed first-level PDFs provide 600-DPI faces, with explicit app/card mappings and recorded source editions. The SD-15C face belongs to SD-15C, not SD1-15A. Atlas/catalog invariants are updated to the complete imported corpus.\n').encode()
 report={'catalog_entries':len(refs),'native_imported_cards':len(entries),'recorded_tracks':tracks,'atlas_rows':len(rowmap),'source_faces':plan['source_faces'],'branch':args.branch,'validation_scope':'Container/checksum/source preservation and extraction checks performed by Python. Run owner Rust gate before merge.'}
 files[Path('programs/HP67/ImportedPACs/installation.json')]=json.dumps(report,indent=2).encode()
 files[Path('docs/PAC_IMPORT_V2.md')]=('# Local PAC import v2.0\n\n'+json.dumps(report,indent=2)+'\n\nAll artwork comes exclusively from the hash-verified PDF files at the top level of HP67PACS. Original source crop pixels are preserved in the local review output. Generic Lambert/Mercator faces are shared by the corresponding zone datasets; zone names remain catalog metadata. Filename revision letters are preserved separately from the PDF face edition. Supplemental Dave Eaton packs do not have supplied PAC PDFs: no scanned face is invented.\n').encode()
 # Fail before branch creation if an untracked output would be overwritten.
 tracked=set(run(repo,'ls-files').splitlines())
 for rel in files:
  need(not (repo/rel).exists() or rel.as_posix() in tracked,f'Untracked file would be overwritten: {repo/rel}')
 for p in (work/'payload').rglob('*'):
  if p.is_file():
   rel=p.relative_to(work/'payload');dst=repo/rel
   if dst.exists() and rel!=Path('assets/hp67-card-artwork-atlas.png'):raise ValueError(f'Import output already exists: {dst}')
 run(repo,'switch','-c',args.branch)
 backup={};created=[]
 try:
  for rel,data in files.items():
   dst=repo/rel
   if dst.exists():backup[dst]=dst.read_bytes()
   else:created.append(dst)
   dst.parent.mkdir(parents=True,exist_ok=True);dst.write_bytes(data)
  for p in (work/'payload').rglob('*'):
   if not p.is_file():continue
   dst=repo/p.relative_to(work/'payload')
   if dst.exists():backup[dst]=dst.read_bytes()
   else:created.append(dst)
   dst.parent.mkdir(parents=True,exist_ok=True);copy_checked(p,dst,plan['payload_hashes'].get(p.relative_to(work/'payload').as_posix()))
  for rel,digest in plan['payload_hashes'].items():need(sha(repo/rel)==digest,f'Installed output differs: {rel}')
  writejson(work/'installation.json',report)
 except BaseException:
  for p,b in backup.items():p.write_bytes(b)
  for p in reversed(created):p.unlink(missing_ok=True)
  raise
 print('INSTALLED '+json.dumps(report),flush=True)
 print('Next: run gate_v2_0.ps1. No commit, push or merge was performed.',flush=True)

def verify(args):
 repo=args.repo.resolve();base=repo/'programs/HP67/ImportedPACs'
 manifest=json.loads((base/'import_manifest.json').read_text(encoding='utf-8'))
 report=json.loads((base/'installation.json').read_text(encoding='utf-8'))
 for e in manifest['entries']:
  p=repo/e['file'];need(sha(p)==e['sha256'],f'Media hash: {p}');need(validate_card(p)==e['tracks'],f'Tracks: {p}')
  if e['face']:
   p=repo/e['face']['path']
   with Image.open(p) as im:
    need(im.size==art.card_pixel_size(600),f'Artwork size: {p}')
    need(im.convert('RGBA').getchannel('A').getextrema()==(255,255),f'Artwork alpha: {p}')
 for p in manifest['pdfs'].values():need(sha(base/'manuals'/p['file'])==p['sha256'],f'Manual hash: {p["file"]}')
 with Image.open(repo/'assets/hp67-card-artwork-atlas.png') as atlas:
  need(atlas.size==(499,report['atlas_rows']*80),'Atlas dimensions')
  need(atlas.convert('RGBA').getchannel('A').getextrema()==(255,255),'Atlas alpha')
 text=(repo/'src/program_library.rs').read_text(encoding='utf-8');_,_,blocks=catalog_blocks(text)
 need(len(blocks)==report['catalog_entries'],'Catalog count')
 refs=[re.search(r'reference: "([^"]+)"',b)[1] for b in blocks];need(len(refs)==len(set(refs)),'Duplicate catalog refs')
 need(all(e['reference'] in refs for e in manifest['entries']),'Imported references missing')
 print('VERIFY OK: '+json.dumps(report),flush=True)

def main():
 p=argparse.ArgumentParser(description=__doc__);s=p.add_subparsers(dest='command',required=True)
 a=s.add_parser('prepare');a.add_argument('--source-root',type=Path,required=True);a.add_argument('--work',type=Path,required=True);a.set_defaults(func=prepare)
 a=s.add_parser('install');a.add_argument('--repo',type=Path,required=True);a.add_argument('--work',type=Path,required=True);a.add_argument('--branch',default='agent/import-all-pacs-local-v2');a.set_defaults(func=install)
 a=s.add_parser('verify');a.add_argument('--repo',type=Path,required=True);a.set_defaults(func=verify)
 a=p.parse_args()
 try:a.func(a)
 except (ValueError,OSError,KeyError) as e:print('ERROR:',e,file=sys.stderr);return 2
 return 0
if __name__=='__main__':raise SystemExit(main())

