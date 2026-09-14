"""Photo / before / after audit of every panel legend and both faces of every key.

Usage: python tools/visual_compare/symbol_audit.py target/visual/spacing-final
The photo is locally cropped at measured row/column anchors. Its aspect ratio
is preserved and scale follows nearby key width; this is not a pixelwise score.
"""
import csv
import html
import json
import math
import sys
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).parent
out = Path(sys.argv[1])
photo = Image.open(ROOT / 'reference.jpg').convert('RGB')
before = Image.open('target/visual/matrix-final/1320x2480.png').convert('RGB')
after = Image.open(out / '1320x2480.png').convert('RGB')
with (out / 'legends.csv').open() as f:
    legends = list(csv.DictReader(f))
with (out / 'keys.csv').open() as f:
    keys = list(csv.DictReader(f))

# Pixel anchors in the unmodified Commons photograph (768 x 1136).
PHOTO_ROWS = [283, 373, 456, 545, 635, 725, 819, 917, 1017]
KEY_TOPS = [302, 386, 474, 562, 653, 747, 841, 940]
DESIGN_TOPS = [169, 223, 277, 331, 383, 435, 487, 538]

def reference_x(row, x):
    if row < 4:
        return 223 + (x - 58) * 1.64
    if row == 4:
        return {56: 214, 119: 318, 166: 400, 220: 490, 274: 580}[x]
    return {56: 208, 119: 316, 194: 450, 270: 584}[x]

def native_crop(im, x, y, w, h):
    return im.crop(tuple(round(v * 4) for v in (x-w/2, y-h/2, x+w/2, y+h/2)))

def photo_crop(x, y, w, h, scale):
    return photo.crop(tuple(round(v) for v in (x-w*scale/2, y-h*scale/2, x+w*scale/2, y+h*scale/2))).resize((round(w*4), round(h*4)), Image.Resampling.LANCZOS)

def sheet(name, cards, columns, width, height):
    im = Image.new('RGB', (columns * width, math.ceil(len(cards)/columns)*height), '#eee')
    d = ImageDraw.Draw(im)
    for i, (label, images) in enumerate(cards):
        x, y = (i % columns)*width, (i//columns)*height
        d.text((x+5,y+3), label, fill='#111')
        for j, (title, crop) in enumerate(images):
            crop.thumbnail((width-55,(height-20)//len(images)-2), Image.Resampling.LANCZOS)
            top = y+20+j*((height-20)//len(images))
            d.text((x+3,top+3), title, fill='#333')
            im.paste(crop,(x+51,top))
    im.save(out/name)

cards=[]
for cell in legends:
    row=int(cell['row']); x=float(cell['center_x']); y=float(cell['center_y']); w=float(cell['width'])
    ref = photo_crop(reference_x(row,x),PHOTO_ROWS[row],w,17,1.65 if row<4 else 2.0)
    cards.append((cell['mark'], [('Photo',ref),('Before',native_crop(before,x,y,w,17)),('After',native_crop(after,x,y,w,17))]))
sheet('legend-audit.png',cards,3,310,155)

cards=[]
for key in keys:
    x=float(key['center_x']); y=float(key['top']); w=float(key['width']); h=float(key['height'])
    row=DESIGN_TOPS.index(int(y))
    if row < 3:
        px=220+(x-58)*1.65; scale=1.65; ph=52
    elif row==3:
        px=260 if key['key']=='enter' else 400+(x-166)*1.65; scale=1.8; ph=54
    else:
        px={56:204,119:315,194:449,270:583}[x]; scale=2.0; ph=56
    # Wider crop height preserves the actual photo key silhouette, not just text.
    ref=photo_crop(px,KEY_TOPS[row]+ph/2,w+5,(h+5),scale)
    cards.append((key['key'], [('Photo',ref),('After',native_crop(after,x,y+h/2,w+5,h+5))]))
sheet('key-audit.png',cards,5,190,210)

def color_gap(im, box):
    bounds=[]
    for blue in [False, True]:
        xs=[]
        for y in range(box[1],box[3]):
            for x in range(box[0],box[2]):
                r,g,b=im.getpixel((x,y))
                match=(g>r*1.15 and b>g*1.08 and b>150) if blue else (r>110 and g>110 and b<g*.9 and r>g*.83)
                if match: xs.append(x)
        bounds.append((min(xs),max(xs)))
    return bounds[1][0]-bounds[0][1]-1

measurements=[]
for label, box, logical, render_box in [
    ('LN / e^x',(277,708,360,737),11.5,(89,416,150,433)),
    ('LOG / 10^x',(400,708,499,737),10.5,(161,416,227,433)),
    ('sqrt(x) / x^2',(540,708,626,737),10.5,(237,416,303,433)),
    ('% / %CH',(272,1005,365,1030),10.0,(88,570,150,583)),
    ('INT / FRAC',(399,1005,510,1030),10.5,(161,570,227,583)),
]:
    gap=color_gap(photo,box)
    raster_gap=color_gap(after,tuple(v*4 for v in render_box))/4
    assert abs(raster_gap-logical)<1.0, (label,raster_gap,logical)
    measurements.append(dict(label=label,photo_crop=box,photo_gap_px=gap,nearby_key_width_px=72,photo_gap_per_key=round(gap/72,3),renderer_gap_logical=logical,renderer_gap_per_key=round(logical/36,3),measured_render_gap=raster_gap))
(out/'spacing-measurements.json').write_text(json.dumps(measurements,indent=2)+'\n')
rows=''.join('<tr>'+''.join('<td>'+html.escape(str(m[k]))+'</td>' for k in ('label','photo_gap_px','photo_gap_per_key','renderer_gap_logical','renderer_gap_per_key','measured_render_gap'))+'</tr>' for m in measurements)
(out/'symbol-report.html').write_text('''<!doctype html><meta charset="utf-8"><title>HP-67 symbol and spacing audit</title>
<style>body{font:16px system-ui;background:#faf9f6;color:#222;margin:30px}img{max-width:100%}td,th{padding:8px;border:1px solid #aaa}table{border-collapse:collapse}</style>
<h1>HP-67 symbol and spacing audit</h1><p>Photo / before / after: all 38 panel groups. Every key is also shown with both printed faces. Photo crops preserve aspect ratio and use nearby key-width scale. Raised keys retain photographic parallax.</p>
<p>Five yellow/cyan gaps measured by color separation. Pixel thresholds and blur give approximately +/-2 px uncertainty. These measurements establish space between groups, not font identity or a perfect photographic match.</p>
<table><tr><th>Group</th><th>Photo gap (px)</th><th>Photo gap / key width</th><th>Renderer gap (logical)</th><th>Renderer gap / key width</th><th>Measured render gap</th></tr>'''+rows+'''</table>
<p>Arrow audit: x/y, x/I and P/S have compact heads. R/P, D/R and H/H.MS have full shafts, cyan upper/right and yellow lower/left. The top R-down and ENTER have shafts; the 8/9 skirts have triangles.</p>
<h2>Panel legends</h2><img src="legend-audit.png"><h2>Keys</h2><img src="key-audit.png">''',encoding='utf-8')
print(f'Audited {len(legends)} panel groups and {len(keys)} keys; {out / "symbol-report.html"}')
