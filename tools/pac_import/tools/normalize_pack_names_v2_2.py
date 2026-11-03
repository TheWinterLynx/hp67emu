"""Normalize presentation names without changing magnetic identities or source paths."""
import argparse
import json
import re
from pathlib import Path

NAMES = {
    'Demo Pac 1': 'HP-67 Demonstration Pac I',
    'HP-67 Games Pac 1': 'HP-67 Games Pac I',
    'HP-67 Games Pac I': 'HP-67 Games Pac I',
    'HP-67 Standard Pac': 'HP-67 Standard Pac',
    'HP-67 Diagnostic Cards': 'HP-67 Diagnostic Pac',
    'Custom Diagnostic Pacs': 'HP-67 Custom Diagnostic Pac',
    'Business Decisions Pac': 'HP-67 Business Decisions Pac I',
    'HP-67 Civil Engineering Pac I': 'HP-67 Civil Engineering Pac I',
    'HP-67 EE Pac I': 'HP-67 Electrical Engineering Pac I',
    'HP-67 Math Pac I': 'HP-67 Mathematics Pac I',
    'HP-67 ME Pac': 'HP-67 Mechanical Engineering Pac I',
    'HP-67 Stat Pac I': 'HP-67 Statistics Pac I',
    'HP-67 Clinical Lab and Nuclear Medicine Pac': 'HP-67 Clinical Laboratory and Nuclear Medicine Pac',
    'HP-67 Navigation Pac I': 'HP-67 Navigation Pac I',
    'HP-67 Surveying Pac I': 'HP-67 Surveying Pac I',
    'Dave Eaton HP-67': 'HP-67 Dave Eaton Pac',
    'Dave Eaton HP-67/97 2026': 'HP-67 Dave Eaton Supplementary Pac (2026)',
}

TEST = '''    #[test]
    fn pack_names_use_consistent_hp67_prefix() {
        for entry in PROGRAM_LIBRARY {
            assert!(entry.pack.starts_with("HP-67 "), "{}", entry.pack);
        }
    }

'''

def normalize(text):
    before = re.findall(r'pack: "([^"]+)"', text)
    unknown = set(before) - set(NAMES) - set(NAMES.values())
    if unknown:
        raise ValueError(f'Unknown pack names: {sorted(unknown)}')
    text = re.sub(r'pack: "([^"]+)"', lambda m: 'pack: '+json.dumps(NAMES.get(m[1],m[1])), text)
    text = text.replace('entry.pack == "Custom Diagnostic Pacs"', 'entry.pack == "HP-67 Custom Diagnostic Pac"')
    old_import = 'use crate::ui::program_card::{ProgramCardArtwork, MOON_ROCKET_LANDER_CARD};'
    if old_import in text and text.count('MOON_ROCKET_LANDER_CARD') == 1:
        text = text.replace(old_import, 'use crate::ui::program_card::ProgramCardArtwork;')
    if 'fn pack_names_use_consistent_hp67_prefix()' not in text:
        marker = 'mod tests {\n    use super::*;\n\n'
        if marker not in text: raise ValueError('Catalog tests module missing')
        text = text.replace(marker, marker + TEST, 1)
    return text


def main():
    p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);a=p.parse_args()
    path=a.repo/'src/program_library.rs';old=path.read_text(encoding='utf-8');new=normalize(old)
    # Media include paths, references and artwork mappings must be byte-identical.
    for pattern in [r'reference: "[^"]+"',r'"/?programs/[^"\n]+"',r'"[^"\n]+" => Some\(\d+\)']:
        if re.findall(pattern,old)!=re.findall(pattern,new):raise ValueError('Non-presentation change rejected')
    path.write_text(new,encoding='utf-8')
    manifest=a.repo/'programs/HP67/ImportedPACs/import_manifest.json'
    if manifest.exists():
        data=json.loads(manifest.read_text(encoding='utf-8'))
        for e in data['entries']:
            if 'pack' in e:e['pack']=NAMES.get(e['pack'],e['pack'])
        manifest.write_text(json.dumps(data,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
    print('PACK NAMES OK: '+', '.join(dict.fromkeys(re.findall(r'pack: "([^"]+)"',new))),flush=True)
if __name__=='__main__':main()
