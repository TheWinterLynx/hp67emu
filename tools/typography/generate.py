"""Generate vector-only glyph meshes from OFL Arimo. Analysis dependency: fonttools.

python -m pip install --target target/fonttools fonttools
python tools/typography/generate.py target/Arimo.ttf
Contours are flattened in font space and scan-converted into vector trapezoids,
including counters/holes. No pixels, texture atlas or system font at runtime.
"""
import sys, struct
from pathlib import Path
sys.path.insert(0, str(Path('target/fonttools').resolve()))
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont
from fontTools.pens.basePen import BasePen


class Contours(BasePen):
    def __init__(self, glyphset):
        super().__init__(glyphset)
        self.paths = []
        self.current = []

    def _moveTo(self, p): self.current = [p]
    def _lineTo(self, p): self.current.append(p)
    def _closePath(self):
        self.current.append(self.current[0])
        self.paths.append(self.current)
    def _qCurveToOne(self, p1, p2):
        p0 = self.current[-1]
        for i in range(1, 13):
            t = i/12
            self.current.append(tuple((1-t)**2*a+2*(1-t)*t*b+t*t*c for a,b,c in zip(p0,p1,p2)))
    def _curveToOne(self, p1, p2, p3):
        p0 = self.current[-1]
        for i in range(1, 17):
            t=i/16
            self.current.append(tuple((1-t)**3*a+3*(1-t)**2*t*b+3*(1-t)*t*t*c+t**3*d for a,b,c,d in zip(p0,p1,p2,p3)))


chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 .,!%?()+-−/=<>:;'
binary=bytearray()
for weight in [400, 600, 700]:
    font = instantiateVariableFont(TTFont(sys.argv[1]), {'wght':weight})
    gs = font.getGlyphSet()
    cmap = font.getBestCmap()
    cap = font['OS/2'].sCapHeight
    for ch in chars:
        glyph = gs[cmap[ord(ch)]]
        pen = Contours(gs)
        glyph.draw(pen)
        edges = [(a,b) for path in pen.paths for a,b in zip(path,path[1:]) if abs(a[1]-b[1])>1e-6]
        levels = sorted(set(round(p[1],5) for path in pen.paths for p in path))
        triangles = []
        for lo,hi in zip(levels,levels[1:]):
            if hi-lo < 1e-5: continue
            mid=(lo+hi)/2
            crossing=[e for e in edges if min(e[0][1],e[1][1])<mid<max(e[0][1],e[1][1])]
            def x(e,y):
                a,b=e
                return a[0]+(y-a[1])*(b[0]-a[0])/(b[1]-a[1])
            crossing.sort(key=lambda e:x(e,mid))
            assert len(crossing)%2==0
            for left,right in zip(crossing[::2],crossing[1::2]):
                a=(x(left,lo)/cap,1-lo/cap); b=(x(right,lo)/cap,1-lo/cap)
                c=(x(left,hi)/cap,1-hi/cap); d=(x(right,hi)/cap,1-hi/cap)
                triangles.extend([a,b,c,b,d,c])
        vertices=[]; indices=[]; seen={}
        for xy in triangles:
            xy=tuple(round(v,6) for v in xy)
            if xy not in seen:
                seen[xy]=len(vertices); vertices.append(xy)
            indices.append(seen[xy])
        binary.extend(struct.pack('<HIfII',weight,ord(ch),glyph.width/cap,len(vertices),len(indices)))
        for xy in vertices: binary.extend(struct.pack('<ff',*xy))
        for i in indices: binary.extend(struct.pack('<H',i))
        binary.extend(struct.pack('<H',len(pen.paths)))
        for path in pen.paths:
            path=path[:-1]
            binary.extend(struct.pack('<H',len(path)))
            for x,y in path: binary.extend(struct.pack('<ff',x/cap,1-y/cap))
Path('src/ui/lettering.bin').write_bytes(binary)
print(f'{len(binary)} bytes of vector coordinates and triangle indices')
