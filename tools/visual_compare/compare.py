"""Register a photograph, then compare actual egui mesh captures. Pillow only.

python tools/visual_compare/compare.py target/visual/baseline
"""
import json
import sys
from pathlib import Path
from PIL import Image, ImageChops, ImageFilter, ImageDraw, ImageStat

ROOT = Path(__file__).parent


def solve(rows):
    for i in range(8):
        pivot = max(range(i, 8), key=lambda j: abs(rows[j][i]))
        rows[i], rows[pivot] = rows[pivot], rows[i]
        d = rows[i][i]
        rows[i] = [v / d for v in rows[i]]
        for j in range(8):
            if j != i:
                d = rows[j][i]
                rows[j] = [a - d*b for a, b in zip(rows[j], rows[i])]
    return [r[-1] for r in rows]


def register(photo, size, data):
    rows = []
    for (x, y), (u, v) in zip(data['panel_logical'], data['panel_photo']):
        x *= size[0]/330
        y *= size[1]/620
        rows += [[x,y,1,0,0,0,-u*x,-u*y,u], [0,0,0,x,y,1,-v*x,-v*y,v]]
    return photo.transform(size, Image.Transform.PERSPECTIVE, solve(rows), Image.Resampling.BICUBIC)


def main():
    out = Path(sys.argv[1])
    data = json.loads((ROOT/'measurements.json').read_text())
    photo = Image.open(ROOT/'reference.jpg').convert('RGB')
    metrics = {}
    for size in [(330,620), (660,1240)]:
        name = f'{size[0]}x{size[1]}'
        render = Image.open(out/(name+'.ppm')).convert('RGB')
        render.save(out/(name+'.png'))
        reference = register(photo,size,data)
        reference.save(out/(name+'-reference.png'))
        Image.blend(reference,render,0.5).save(out/(name+'-overlay.png'))
        diff = ImageChops.difference(reference,render)
        diff.save(out/(name+'-difference.png'))
        edge = ImageChops.difference(reference.convert('L').filter(ImageFilter.FIND_EDGES), render.convert('L').filter(ImageFilter.FIND_EDGES))
        edge.save(out/(name+'-edges.png'))
        guides = Image.blend(reference,render,0.5)
        draw = ImageDraw.Draw(guides)
        s=size[0]/330
        for y in data['row_tops']:
            draw.line((35*s,y*s,295*s,y*s), fill='magenta',width=1)
        for x in data['function_centers']:
            draw.line((x*s,140*s,x*s,365*s),fill='lime',width=1)
        for x in data['numeric_centers']:
            draw.line((x*s,380*s,x*s,580*s),fill='cyan',width=1)
        guides.save(out/(name+'-guides.png'))
        side=Image.new('RGB',(size[0]*2,size[1])); side.paste(reference); side.paste(render,(size[0],0)); side.save(out/(name+'-side-by-side.png'))
        metrics[name]={'rgb_mae':sum(ImageStat.Stat(diff).mean)/3,'edge_mae':ImageStat.Stat(edge).mean[0]}
    (out/'metrics.json').write_text(json.dumps(metrics,indent=2)+'\n')
    print(json.dumps(metrics,indent=2))


if __name__ == '__main__':
    main()
