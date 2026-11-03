"""Repair imported PAC artwork from preserved root-PDF crops; no program changes."""
import argparse, json, re, shutil, hashlib, io
from pathlib import Path
from PIL import Image, ImageDraw
import artwork_v2_1 as art

def save_png(image,path):
    stream=io.BytesIO();image.save(stream,format='PNG',dpi=(600,600));data=stream.getvalue()
    path.write_bytes(data)
    if path.read_bytes()!=data:raise OSError(f'Incomplete PNG write: {path}')

def render(source):
    im=Image.open(source).convert('RGBA');w,h=art.card_pixel_size(600)
    scale=min(w/im.width,h/im.height)
    im=im.resize((min(w,round(im.width*scale)),min(h,round(im.height*scale))),Image.Resampling.LANCZOS)
    im.putalpha(art.card_silhouette_mask((w,h),600).resize(im.size,Image.Resampling.NEAREST))
    return art.normalize_card_artwork(im,600)

def main():
    p=argparse.ArgumentParser();p.add_argument('--work',type=Path,required=True);p.add_argument('--repo',type=Path,required=True);a=p.parse_args()
    review=a.work/'review';sources=sorted(review.glob('*/*-source.png'))
    if len(sources)!=232:raise ValueError(f'Expected 232 preserved source crops, found {len(sources)}')
    text=(a.repo/'src/program_library.rs').read_text(encoding='utf-8')
    manifest=json.loads((a.repo/'programs/HP67/ImportedPACs/import_manifest.json').read_text(encoding='utf-8'))
    blocks=re.findall(r'ProgramLibraryEntry\s*\{(.*?)\n    \}',text,re.S)
    # Match the actual catalog struct name from field content instead of assuming it.
    bindings={}
    for m in re.finditer(r'reference:\s*"([^"]+)"(?:(?!reference:).)*?artwork_path:\s*Some\("([^"]+)"\)',text,re.S):bindings[m[1]]=m[2].lstrip('/')
    rows=[(r,int(n)) for r,n in re.findall(r'"([^"]+)" => Some\((\d+)\)',text)]
    originals={s.name.removesuffix('-source.png'):s for s in sources}
    wanted={e['face']['path'] for e in manifest['entries'] if e.get('face')}
    wanted.update(v for k,v in bindings.items() if k=='SD-15C')
    if not wanted:raise ValueError('No imported artwork bindings')
    for path in wanted:
        if Path(path).stem not in originals:raise ValueError(f'Missing crop: {path}')
        if not (a.repo/path).is_file():raise ValueError(f'Missing installed artwork: {path}')
    if any(r not in bindings for r,n in rows):raise ValueError('Atlas row without catalog artwork')
    backup=a.repo/'artwork-backup-v2.2'
    if backup.exists():raise ValueError(f'Backup exists; no changes made: {backup}')
    output=a.work/'review-v2.2'
    if output.exists():raise ValueError(f'Review already exists: {output}')
    atlas=Image.open(a.repo/'assets/hp67-card-artwork-atlas.png').convert('RGBA')
    if atlas.size!=(499,len(rows)*80):raise ValueError('Unexpected atlas dimensions')
    # Render everything before writing installed assets.
    images={}
    for i,(key,source) in enumerate(originals.items(),1):
        images[key]=render(source)
        if i%20==0 or i==len(originals):print(f'Rendering {i}/{len(originals)}',flush=True)
    for ref,n in rows:
        path=bindings[ref]
        if path in wanted:atlas.paste(images[Path(path).stem].resize((499,80),Image.Resampling.BOX),(0,n*80))
    targets=[a.repo/x for x in sorted(wanted)]+[a.repo/'assets/hp67-card-artwork-atlas.png']
    for target in targets:
        b=backup/target.relative_to(a.repo);b.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(target,b)
    try:
        for path in sorted(wanted):save_png(images[Path(path).stem],a.repo/path)
        save_png(atlas,targets[-1])
        for key in sorted({s.parent.name for s in sources}):
            selected=[s for s in sources if s.parent.name==key]
            sheet=Image.new('RGB',(1340,((len(selected)+1)//2)*145),'#cccccc');draw=ImageDraw.Draw(sheet)
            for i,s in enumerate(selected):
                im=images[s.name.removesuffix('-source.png')].convert('RGB');im.thumbnail((650,105));x=(i%2)*670;y=(i//2)*145
                draw.text((x+8,y+4),s.stem,fill='black');sheet.paste(im,(x+8,y+25))
            output.mkdir(parents=True,exist_ok=True);sheet.save(output/(key+'.jpg'),quality=97)
        (output/'index.html').write_text('<!doctype html><meta charset="utf-8"><h1>Tarjetas corregidas v2.1</h1>'+''.join(f'<h2>{key}</h2><img style="max-width:100%" src="{key}.jpg">' for key in sorted({s.parent.name for s in sources})),encoding='utf-8')
        for path in wanted:
            with Image.open(a.repo/path) as im:
                assert im.size==art.card_pixel_size(600) and im.getchannel('A').getextrema()==(255,255)
    except BaseException:
        for target in targets:shutil.copy2(backup/target.relative_to(a.repo),target)
        raise
    print(f'OK: {len(images)} source faces regenerated; {len(wanted)} installed images; {len(rows)} atlas rows. Programs unchanged.')
    print(f'Review: {output / "index.html"}')
    print(f'Backup: {backup}')
if __name__=='__main__':main()
