"""Generate a simplified mechanical STEP model for OK-23GF030-04.

The XY envelope and contact locations follow JLCPCB/EasyEDA part C9900040133.
The housing details are deliberately simplified; see README.md.
"""

from pathlib import Path

import cadquery as cq


OUT = Path(__file__).parent / "OK-23GF030-04.3dshapes" / "CONN-SMD_30P-OK-23GF030-04.step"


def box_at(length: float, width: float, height: float, x: float, y: float, z0: float):
    return (
        cq.Workplane("XY")
        .box(length, width, height)
        .translate((x, y, z0 + height / 2))
    )


# Simplified black insulating housing, 9.10 x 2.85 x 0.65 mm overall.
housing = box_at(9.10, 2.85, 0.10, 0, 0, 0)
housing = housing.union(box_at(0.75, 2.85, 0.55, -4.175, 0, 0.10))
housing = housing.union(box_at(0.75, 2.85, 0.55, 4.175, 0, 0.10))
housing = housing.union(box_at(7.60, 0.35, 0.55, 0, -1.25, 0.10))
housing = housing.union(box_at(7.60, 0.35, 0.55, 0, 1.25, 0.10))

# Visible metal contacts. Pad centers exactly match the generated footprint.
contacts = None
for row_y in (-1.20, 1.20):
    for index in range(15):
        contact = box_at(0.20, 0.50, 0.07, -2.80 + index * 0.40, row_y, 0.10)
        contacts = contact if contacts is None else contacts.union(contact)

# Four hold-down fittings, matching the unnumbered footprint pads.
fittings = None
for x in (-3.63, 3.63):
    for y in (-1.06, 1.06):
        fitting = box_at(0.60, 0.785, 0.08, x, y, 0)
        fittings = fitting if fittings is None else fittings.union(fitting)

assembly = cq.Compound.makeCompound(
    [housing.val(), contacts.val(), fittings.val()]
)
OUT.parent.mkdir(parents=True, exist_ok=True)
cq.exporters.export(assembly, str(OUT))
print(OUT)
