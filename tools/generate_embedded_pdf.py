"""Rebuild/check native embedded-PDF and program-section catalogs from reviewed import metadata."""
import argparse,json,re
from pathlib import Path

def catalogs(root):
    profiles=json.loads((root/'tools/pac_import/tools/profiles-v2.0.json').read_text(encoding='utf-8'))
    manifest=json.loads((root/'programs/HP67/ImportedPACs/import_manifest.json').read_text(encoding='utf-8'))
    names=sorted(['hp67-oh-es.pdf']+[p['file'] for p in profiles.values()])
    if len(names)!=12 or len(set(names))!=12:raise ValueError('Expected 12 distinct manuals')
    links=[]
    for entry in manifest['entries']:
        if not entry.get('pdf'):continue
        key=Path(entry['file']).parent.name
        rows=[row for row in profiles[key]['rows'] if row['app']==entry['app']]
        if not rows:raise ValueError(f'Missing reviewed application: {entry["reference"]}')
        links.append((entry['reference'],Path(entry['pdf']).name,min(row['page'] for row in rows)))
    links.append(('SD-15C',profiles['standard']['file'],profiles['standard']['rows'][-1]['page']))
    if len({row[0] for row in links})!=len(links):raise ValueError('Duplicate program links')
    return names,sorted(links)

def main():
    p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,default=Path(__file__).resolve().parents[1]);p.add_argument('--check',action='store_true');a=p.parse_args();root=a.repo.resolve()
    names,links=catalogs(root)
    for name in names:
        with (root/'documentation'/name).open('rb') as stream:
            if stream.read(5)!=b'%PDF-':raise ValueError(f'Invalid source PDF: {name}')
    embedded=root/'documentation/embedded.rs';sections=root/'documentation/sections.rs'
    if a.check:
        actual=re.findall(r'\(\s*"([^"]+)"\s*,\s*include_bytes!\("([^"]+)"\)\s*,?\s*\)',embedded.read_text())
        if sorted(actual)!=[(name,name) for name in names]:raise ValueError('Embedded PDF catalog is stale')
        actual=[(ref,file,int(page)) for ref,file,page in re.findall(r'\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*(\d+)\s*,?\s*\)',sections.read_text())]
        if actual!=links:raise ValueError('Program section catalog is stale')
    else:
        embedded.write_text('// Original PDF bytes are linked into the desktop executable.\nconst EMBEDDED_PDFS: &[(&str, &[u8])] = &[\n'+''.join(f'    ("{name}", include_bytes!("{name}")),\n' for name in names)+'];\n',encoding='utf-8')
        sections.write_text('// Physical PDF pages, 1-based, from reviewed root-PDF application profiles.\nconst PROGRAM_SECTIONS: &[(&str, &str, usize)] = &[\n'+''.join(f'    ("{ref}", "{file}", {page}),\n' for ref,file,page in links)+'];\n',encoding='utf-8')
    print(f'EMBEDDED CATALOG OK: {len(names)} PDFs; {len(links)} program section links')
if __name__=='__main__':main()
