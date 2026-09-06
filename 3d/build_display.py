"""Reconstruct the supplied display drawings; dimensions in millimetres.

Run with CadQuery 2.8: python build_display.py
Rear-view pixel tracing is reflected into the front-view coordinate system.
"""
from pathlib import Path
import json
import cadquery as cq

OUT = Path(__file__).resolve().parent / 'output'
OUT.mkdir(exist_ok=True)
W, H, R = 26.50, 82.45, 1.80
FRONT_EDGE_R = .50
LENS, OCA, PANEL = 1.10, .175, .63
REAR = -(LENS + OCA + PANEL)
SCALE = 82.45 / (638 - 90)
parts = []
assembly = cq.Assembly(name='Display_3_189_inch_with_flat_FPC')


def rounded(w, h, r, z, t, x=0, y=0):
    p = cq.Workplane('XY').box(w, h, t, centered=(True, True, False))
    if r:
        p = p.edges('|Z').fillet(r)
    return p.translate((x, y, z))


def add(name, shape, color):
    assert shape.val().isValid(), name
    assert shape.val().Volume() > 0, name
    parts.append((name, shape, color))
    assembly.add(shape, name=name, color=cq.Color(*color))


def xy(px, py):
    return ((433.25-px)*SCALE, -H/2+(638-py)*SCALE)


def traced(points, z, thickness):
    return cq.Workplane('XY').polyline([xy(*p) for p in points]).close().extrude(thickness).translate((0, 0, z))


def pixel_box(x0, y0, x1, y1, z, thickness):
    x, y = xy((x0+x1)/2, (y0+y1)/2)
    return rounded((x1-x0)*SCALE, (y1-y0)*SCALE, 0, z, thickness, x, y)


# Split the last 0.02 mm of the lens to preserve the visible black border in STEP.
# Fillet the complete lens first so the rounding crosses the cosmetic split.
lens = rounded(W, H, R, -LENS, LENS).faces('>Z').edges().fillet(FRONT_EDGE_R)
front_slice = rounded(W+2, H+2, 0, -.02, .02)
add('Cover_lens_26_50_x_82_45_R1_80', lens.cut(front_slice), (.20, .25, .29, .45))
opening = rounded(22.50, 78.45, 1.50, -.02, .02)
border = lens.intersect(front_slice).cut(opening)
add('Black_border_VA_22_50_x_78_45_R1_50', border, (.025, .025, .028))
add('Viewing_window', opening, (.10, .16, .20, .55))
# Back view gives 23.81 x 81.05. Bottom aligned; resulting top inset is 1.40.
PW, PH = 23.81, 81.05
PY = -(H-PH)/2
add('Optical_adhesive_0_175', rounded(PW, PH, 0, -LENS-OCA, OCA, y=PY), (.55, .65, .70, .35))
add('LCD_panel_0_63', rounded(PW, PH, 0, REAR, PANEL, y=PY), (.35, .38, .40))

# One continuous flat FPC silhouette, manually traced from the rear drawing.
# The outline includes the main lower board, curved lateral tail, and end pad.
outline = [
    (367,636), (461,636), (461,639), (452,643), (451,648),
    (454,652), (491,660), (493,664), (493,684), (503,694),
    (503,711), (501,714), (497,715), (495,718), (498,721),
    (526,721), (540,717), (558,710), (567,708), (709,708),
    (716,710), (722,715), (725,721), (726,726), (726,749),
    (729,753), (735,753), (737,756), (737,782), (680,782),
    (680,757), (683,753), (689,753), (690,749), (690,745),
    (576,745), (562,748), (545,754), (532,757), (499,757),
    (496,760), (497,763), (501,765), (501,815), (498,818),
    (362,818), (358,815), (358,657), (360,652), (366,645),
]
FLEX_T = .15
add('FPC_flat_estimated_outline_0_15', traced(outline, REAR, FLEX_T), (.76, .40, .065))
# Reinforcement and component envelopes: approximate XY positions from drawings.
add('Rear_upper_PI_stiffener_0_125', pixel_box(359,653,409,697,REAR-.125,.125), (.82,.69,.36))
add('Rear_lower_PI_stiffener_0_125', pixel_box(359,765,454,817,REAR-.125,.125), (.82,.69,.36))
add('Rear_driver_IC_envelope_0_58_max', pixel_box(424,670,487,689,REAR-.58,.58), (.06,.06,.06))
add('Rear_small_component_envelope_estimated', pixel_box(359,725,398,748,REAR-.60,.60), (.07,.07,.07))
add('Rear_lower_component_envelope_estimated', pixel_box(457,776,495,816,REAR-.70,.70), (.07,.07,.07))
add('Front_upper_component_envelope_0_975_max', pixel_box(359,642,409,692,REAR+FLEX_T,.975), (.10,.10,.10))
add('Front_lower_component_envelope_1_225_max', pixel_box(359,765,454,817,REAR+FLEX_T,1.225), (.10,.10,.10))
add('Front_small_driver_envelope_estimated', pixel_box(414,692,486,708,REAR+FLEX_T,.58), (.10,.10,.10))
add('Connector_pad_reinforcement_estimated', pixel_box(680,754,737,782,REAR+FLEX_T,.15), (.85,.73,.42))
add('Connector_OK_23CM030_04_envelope_estimated', pixel_box(684,761,734,777,REAR-1.0,1.0), (.09,.09,.09))
# The drawing calls out a 5 x 5 mm release liner extending past the lower board.
# It is intentionally excluded from assembly geometry because it is removable.

step = OUT / 'display_with_flex.step'
assembly.export(str(step))
compound = cq.Compound.makeCompound([p.val() for _,p,_ in parts])
cq.exporters.export(compound, str(OUT / 'display_with_flex.stl'))
# Verify the exported STEP can be read and retains all solid bodies.
loaded = cq.importers.importStep(str(step))
assert loaded.val().isValid(), 'STEP re-import invalid'
assert len(loaded.solids().vals()) == len(parts), 'Unexpected solid count'
bb = compound.BoundingBox()
report = dict(units='mm', solids=len(parts), step_reimport_valid=True,
              bounds_mm=dict(x=[bb.xmin,bb.xmax], y=[bb.ymin,bb.ymax], z=[bb.zmin,bb.zmax]),
              panel_outline_mm=[W,H], nominal_stack_mm=LENS+OCA+PANEL,
              front_outer_edge_radius_mm=FRONT_EDGE_R,
              flex_thickness_mm=FLEX_T, rear_drawing_mm_per_pixel=SCALE)
(OUT / 'validation.json').write_text(json.dumps(report, indent=2)+'\n')

# Preview rendered directly from the CAD solid tessellations.
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d.art3d import Poly3DCollection
fig = plt.figure(figsize=(12,6), facecolor='#edf0f4')
for index, (title, elevation, azimuth) in enumerate([
    ('Front / flat flex', 90, -90), ('Rear / component envelopes', -90, 90),
    ('Assembly perspective', 35, -65)], 1):
    ax = fig.add_subplot(1,3,index,projection='3d')
    for name, p, color in parts:
        vertices, faces = p.val().tessellate(.08)
        verts = [(v.x,v.y,v.z) for v in vertices]
        mesh = Poly3DCollection([[verts[i] for i in face] for face in faces],
                               facecolor=color[:3], edgecolor='none', linewidth=0)
        ax.add_collection3d(mesh)
    ax.set_xlim(bb.xmin-2,bb.xmax+2)
    ax.set_ylim(bb.ymin-2,bb.ymax+2)
    ax.set_zlim(-6,6)
    ax.set_box_aspect((bb.xlen+4,bb.ylen+4,12))
    ax.view_init(elev=elevation,azim=azimuth)
    ax.set_axis_off()
    ax.set_title(title,fontsize=11)
fig.suptitle('3.189 inch display — 26.50 × 82.45 mm\nEstimated flat FPC and connector geometry',fontsize=15)
fig.tight_layout()
fig.savefig(OUT / 'display_preview.png', dpi=170, facecolor=fig.get_facecolor())
print(json.dumps(report,indent=2))
