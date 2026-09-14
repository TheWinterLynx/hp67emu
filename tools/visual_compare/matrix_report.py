"""Plot the exported production matrix over a native render; no duplicate anchors."""
import csv, html, sys
from pathlib import Path
from PIL import Image, ImageDraw

out=Path(sys.argv[1])
with (out/'legends.csv').open() as f: legends=list(csv.DictReader(f))
with (out/'keys.csv').open() as f: keys=list(csv.DictReader(f))
for file in out.glob('*.ppm'): Image.open(file).save(file.with_suffix('.png'))
im=Image.open(out/'660x1240.png').convert('RGB')
draw=ImageDraw.Draw(im)
for cell in legends:
    x,y,w=(float(cell[k])*2 for k in ('center_x','center_y','width'))
    draw.rectangle((x-w/2,y-16,x+w/2,y+16),outline=(71,144,197))
    draw.line((x-w/2,y,x+w/2,y),fill=(82,202,255))
    draw.line((x,y-5,x,y+5),fill=(255,115,211))
for key in keys:
    x,y,w,h=(float(key[k])*2 for k in ('center_x','top','width','height'))
    draw.rectangle((x-w/2,y,x+w/2,y+h),outline=(127,190,88))
    for name in ('main_center_y','front_center_y'):
        cy=float(key[name])*2
        draw.line((x-5,cy,x+5,cy),fill=(255,173,69))
im.save(out/'matrix-overlay.png')
def table(rows):
    heads=list(rows[0])
    return '<table><thead><tr>'+''.join('<th>'+html.escape(k)+'</th>' for k in heads)+'</tr></thead><tbody>'+''.join('<tr>'+''.join('<td>'+html.escape(str(row[k]))+'</td>' for k in heads)+'</tr>' for row in rows)+'</tbody></table>'
(out/'matrix-report.html').write_text('<!doctype html><meta charset="utf-8"><title>HP-67 layout matrix</title><style>body{font:16px system-ui;margin:30px;background:#faf9f6;color:#222}table{border-collapse:collapse;margin:24px 0}td,th{border:1px solid #ccc;padding:6px 12px;text-align:left}img{max-width:100%}</style><h1>HP-67 production layout matrix</h1><p>Blue: legend cell and center line. Pink: column anchor. Green: key geometry. Orange: key-face print centers. Coordinates are logical units on the 330 x 620 canvas.</p><img src="matrix-overlay.png"><h2>Legend groups</h2>'+table(legends)+'<h2>Key faces</h2>'+table(keys),encoding='utf-8')
