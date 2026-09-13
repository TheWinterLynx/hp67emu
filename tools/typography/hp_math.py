"""Hand-shaped HP mathematical letters in cap-height coordinates, y downward.

Control points follow the supplied close-up's curved x arms and hooked y tail.
These are explicit reconstruction contours, not a substitution with italic text.
"""
import math


def curve(start, *segments):
    points = [start]
    for b, c, d in segments:
        a = points[-1]
        for i in range(1, 25):
            t = i / 24
            points.append(tuple((1-t)**3*a[k]+3*(1-t)**2*t*b[k]+3*(1-t)*t*t*c[k]+t**3*d[k] for k in (0, 1)))
    return points


def stroke(points, thickness):
    left, right = [], []
    for i, (x, y) in enumerate(points):
        a, b = points[max(0, i-1)], points[min(len(points)-1, i+1)]
        dx, dy = b[0]-a[0], b[1]-a[1]
        length = math.hypot(dx, dy)
        # Softly tapered printed terminals, a fuller waist at the crossing.
        width = thickness * (0.85+0.15*math.sin(math.pi*i/(len(points)-1))) / 2
        nx, ny = -dy/length*width, dx/length*width
        left.append((x+nx, y+ny)); right.append((x-nx, y-ny))
    path = left + right[::-1]
    area = sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(path,path[1:]+path[:1]))
    if area < 0: path.reverse()
    return path + path[:1]


def glyph(ch, weight):
    thickness = {400: 0.105, 600: 0.12, 700: 0.135}[weight]
    if ch == 'x':
        return 0.80, [
            stroke(curve((.04,.36), ((.22,.20),(.32,.36),(.39,.61)),
                         ((.47,.88),(.56,1.03),(.73,.92))), thickness),
            stroke(curve((.72,.28), ((.55,.21),(.45,.47),(.33,.68)),
                         ((.20,.91),(.12,1.02),(.02,.93))), thickness),
        ]
    if ch == 'y':
        return 0.79, [
            stroke(curve((.10,.31), ((.20,.25),(.13,.71),(.32,.81)),
                         ((.47,.91),(.59,.48),(.69,.29))), thickness),
            stroke(curve((.69,.29), ((.57,.71),(.40,1.27),(.13,1.23)),
                         ((.07,1.23),(.02,1.21),(.035,1.16))), thickness),
        ]
    if ch == 'π':
        return 0.99, [
            stroke(curve((.03,.43), ((.13,.28),(.39,.35),(.88,.32))), thickness),
            stroke(curve((.28,.35), ((.24,.58),(.26,.88),(.15,1.00))), thickness),
            stroke(curve((.67,.35), ((.60,.69),(.61,1.06),(.84,.92))), thickness),
        ]
    return None
