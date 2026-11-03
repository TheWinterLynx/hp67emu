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

ORDER_HELPER = "/// Stable presentation indices: catalog/media identities never move.\npub fn program_library_display_order() -> &'static [usize] {\n    static ORDER: std::sync::OnceLock<Vec<usize>> = std::sync::OnceLock::new();\n    ORDER.get_or_init(|| {\n        let mut order: Vec<_> = (0..PROGRAM_LIBRARY.len()).collect();\n        order.sort_by_cached_key(|&index| {\n            let entry = &PROGRAM_LIBRARY[index];\n            (entry.pack.to_lowercase(), entry.title.to_lowercase(), entry.reference, index)\n        });\n        order\n    })\n}\n\n"

ORDER_TEST = '    #[test]\n    fn alphabetical_display_order_keeps_every_original_catalog_index() {\n        let order = program_library_display_order();\n        let mut indices = order.to_vec();\n        indices.sort_unstable();\n        assert_eq!(indices, (0..PROGRAM_LIBRARY.len()).collect::<Vec<_>>());\n        for pair in order.windows(2) {\n            let a = &PROGRAM_LIBRARY[pair[0]];\n            let b = &PROGRAM_LIBRARY[pair[1]];\n            assert!((a.pack.to_lowercase(), a.title.to_lowercase(), a.reference)\n                <= (b.pack.to_lowercase(), b.title.to_lowercase(), b.reference));\n        }\n    }\n\n'

def order_library_ui(text):
    if 'let order = crate::program_library::program_library_display_order();' not in text:
        marker='let mut pack_start = 0usize;'
        if text.count(marker)!=3:raise ValueError('Expected three program pack views')
        text=re.sub(r'(?m)^( +)let mut pack_start = 0usize;',lambda m:m[1]+'let order = crate::program_library::program_library_display_order();\n'+m[1]+'let mut pack_start = 0usize;',text)
        text=text.replace('pack_start < PROGRAM_LIBRARY.len()', 'pack_start < order.len()')
        text=text.replace('pack_end < PROGRAM_LIBRARY.len()', 'pack_end < order.len()')
        text=text.replace('PROGRAM_LIBRARY[pack_start].pack', 'PROGRAM_LIBRARY[order[pack_start]].pack')
        text=text.replace('PROGRAM_LIBRARY[pack_end].pack', 'PROGRAM_LIBRARY[order[pack_end]].pack')
        pattern=r'for \((?:index|offset), entry\) in\s*PROGRAM_LIBRARY\[pack_start\.\.pack_end\]\.iter\(\)\.enumerate\(\)\s*\{\s*let index = pack_start \+ (?:index|offset);'
        text,count=re.subn(pattern,'for &index in &order[pack_start..pack_end] {\n let entry = &PROGRAM_LIBRARY[index];',text)
        text=re.sub(r'(?m)^( +)for &index in &order\[pack_start\.\.pack_end\] \{\n let entry = &PROGRAM_LIBRARY\[index\];',lambda m:m[1]+'for &index in &order[pack_start..pack_end] {\n'+m[1]+'    let entry = &PROGRAM_LIBRARY[index];',text)
        if count!=3:raise ValueError('Expected three original-index program loops')
    if 'crate::documentation::help_menu(ui);' in text and 'hp67-help-right' not in text:
        text=re.sub(r'^\s*crate::documentation::help_menu\(ui\);\n','\n',text,flags=re.M)
        marker='                });\n            });\n        });\n        if open_program_library {'
        if text.count(marker)!=1:raise ValueError('Menu bar end anchor changed')
        replacement='                });\n                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {\n                    ui.push_id("hp67-help-right", |ui| crate::documentation::help_menu(ui));\n                });\n            });\n        });\n        if open_program_library {'
        text=text.replace(marker,replacement,1)
    return text

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
    if 'pub fn program_library_display_order()' not in text:
        marker='pub const PROGRAM_LIBRARY: &[ProgramLibraryEntry] = &['
        if text.count(marker)!=1:raise ValueError('Catalog anchor changed')
        text=text.replace(marker,ORDER_HELPER+marker,1)
    if 'fn alphabetical_display_order_keeps_every_original_catalog_index()' not in text:
        marker='mod tests {\n    use super::*;\n\n'
        if marker not in text:raise ValueError('Catalog tests anchor changed')
        text=text.replace(marker,marker+ORDER_TEST,1)
    return text


def main():
    p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);a=p.parse_args()
    path=a.repo/'src/program_library.rs';old=path.read_text(encoding='utf-8');new=normalize(old)
    # Media include paths, references and artwork mappings must be byte-identical.
    for pattern in [r'reference: "[^"]+"',r'"/?programs/[^"\n]+"',r'"[^"\n]+" => Some\(\d+\)']:
        if re.findall(pattern,old)!=re.findall(pattern,new):raise ValueError('Non-presentation change rejected')
    app=a.repo/'src/app.rs';appnew=order_library_ui(app.read_text(encoding='utf-8'))
    path.write_text(new,encoding='utf-8')
    app.write_text(appnew,encoding='utf-8')
    manifest=a.repo/'programs/HP67/ImportedPACs/import_manifest.json'
    if manifest.exists():
        data=json.loads(manifest.read_text(encoding='utf-8'))
        for e in data['entries']:
            if 'pack' in e:e['pack']=NAMES.get(e['pack'],e['pack'])
        manifest.write_text(json.dumps(data,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
    print('PACK NAMES OK: '+', '.join(dict.fromkeys(re.findall(r'pack: "([^"]+)"',new))),flush=True)
if __name__=='__main__':main()

