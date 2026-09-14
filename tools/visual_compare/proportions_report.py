"""Reproducible whole-calculator proportions and final-row clearance audit.

python tools/visual_compare/proportions_report.py target/visual/proportions-final
"""
import csv
import html
import json
import sys
from pathlib import Path
from PIL import Image, ImageDraw
from compare import register, solve

root=Path(__file__).parent
out=Path(sys.argv[1])
source=json.loads((root/'proportion_reference.json').read_text())
profile=json.loads((out/'profile.json').read_text())
def csv_rows(path):
    with path.open() as f: return list(csv.DictReader(f))
keys={r['key']:r for r in csv_rows(out/'keys.csv')}
old={r['key']:r for r in csv_rows(Path('target/visual/spacing-final/keys.csv'))}
legends=csv_rows(out/'legends.csv')

rows=[]
for (x,y),(u,v) in zip(source['photo_corners'],source['logical_corners']):
    rows += [[x,y,1,0,0,0,-u*x,-u*y,u],[0,0,0,x,y,1,-v*x,-v*y,v]]
homography=solve(rows)
def project(point):
    a,b,c,d,e,f,g,h=homography
    x,y=point
    return ((a*x+b*y+c)/(g*x+h*y+1),(d*x+e*y+f)/(g*x+h*y+1))
def projected_size(corners):
    tl,tr,br,bl=map(project,corners)
    return ((tr[0]-tl[0]+br[0]-bl[0])/2,(bl[1]-tl[1]+br[1]-tr[1])/2)

measurements=[]
for name,box in source['key_bounds'].items():
    x0,y0,x1,y1=box
    w,h=projected_size([(x0,y0),(x1,y0),(x1,y1),(x0,y1)])
    kw,kh=float(keys[name]['width']),float(keys[name]['height'])
    ow,oh=float(old[name]['width']),float(old[name]['height'])
    measurements.append(dict(part=name,photo_width=round(w,2),photo_height=round(h,2),before=f'{ow:g} x {oh:g}',after=f'{kw:g} x {kh:g}',photo_ratio=round(w/h,3),before_ratio=round(ow/oh,3),after_ratio=round(kw/kh,3)))
dw,dh=projected_size(source['display_corners'])
measurements.append(dict(part='display',photo_width=round(dw,2),photo_height=round(dh,2),before='269.2 x 49',after=f"{profile['display'][2]} x {profile['display'][3]}",photo_ratio=round(dw/dh,3),before_ratio=round(269.2/49,3),after_ratio=round(profile['display'][2]/profile['display'][3],3)))
last_top=max(float(k['top']) for k in keys.values())
last_keys=sorted((k for k in keys.values() if float(k['top'])==last_top),key=lambda k:float(k['center_x']))
last_legends=[r for r in legends if r['row']=='8']
clearances=[]
for key,legend in zip(last_keys,last_legends):
    gap=float(legend['ink_top'])-float(key['top'])-float(key['height'])
    assert gap>=6.0
    clearances.append(dict(key=key['key'],rest=round(gap,2),held=round(gap-2.8,2)))
ratio=profile['case_height']/profile['case_width']
assert abs(ratio-source['length_mm']/source['width_mm'])<0.00001
data=dict(profile=profile,reference=source,measurements=measurements,last_row_clearance=clearances,case_height_width_ratio=ratio)
(out/'proportions.json').write_text(json.dumps(data,indent=2)+'\n')

photo=Image.open(root/source['photo']).convert('RGB')
ref=register(photo,(660,1240),dict(panel_photo=source['photo_corners'],panel_logical=source['logical_corners']))
ref.save(out/'registered-reference.png')
before=Image.open('target/visual/spacing-final/660x1240.png').convert('RGB')
after=Image.open(out/'660x1240.png').convert('RGB')
sheet=Image.new('RGB',(1980,1270),'#eee');draw=ImageDraw.Draw(sheet)
for x,title,im in [(0,'PHOTO (perspective corrected)',ref),(660,'BEFORE',before),(1320,'AFTER',after)]:
    draw.text((x+10,7),title,fill='black');sheet.paste(im,(x,30))
sheet.save(out/'proportions-comparison.png')
sheet.crop((0,1080,1980,1270)).save(out/'last-row-comparison.png')
overlay=Image.blend(ref,after,0.5);draw=ImageDraw.Draw(overlay)
for key in keys.values():
    cx,y,w,h=(float(key[k])*2 for k in ('center_x','top','width','height'))
    draw.rectangle((cx-w/2,y,cx+w/2,y+h),outline='#5cff6d')
x,y,w,h=(v*2 for v in profile['display']);draw.rectangle((x,y,x+w,y+h),outline='#5cff6d')
overlay.save(out/'proportions-overlay.png')
def table(items):
    names=list(items[0])
    return '<table><tr>'+''.join('<th>'+html.escape(n)+'</th>' for n in names)+'</tr>'+''.join('<tr>'+''.join('<td>'+html.escape(str(row[n]))+'</td>' for n in names)+'</tr>' for row in items)+'</table>'
(out/'proportions-report.html').write_text('''<!doctype html><meta charset="utf-8"><title>HP-67 proportions</title><style>body{font:16px system-ui;background:#faf9f6;color:#222;margin:30px}img{max-width:100%}td,th{border:1px solid #bbb;padding:8px}table{border-collapse:collapse;margin:20px 0}</style><h1>HP-67 whole-calculator proportions</h1><p>Case plan ratio: '''+f'{ratio:.6f}'+''' (152.4 / 81, HP brochure). Internal dimensions below come from the registered photograph; raised keys retain parallax and edge uncertainty. Dimensions are logical units on the 330 x 620 canvas.</p><img src="proportions-comparison.png"><h2>Measured proportions</h2>'''+table(measurements)+'''<h2>Last-row ink clearance</h2><p>Measured from the rendered legend contours to the keycap edge; held values include full 2.8-unit rigid travel.</p>'''+table(clearances)+'''<img src="last-row-comparison.png"><h2>Registration overlay</h2><img src="proportions-overlay.png">''',encoding='utf-8')
print(json.dumps(dict(case_ratio=ratio,display_reference=[dw,dh],last_row_clearance=clearances),indent=2))
