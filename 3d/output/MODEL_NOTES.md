# Display assembly model

`display_with_flex.step` is a millimetre STEP assembly with 16 named, colored solid bodies. `display_preview.png` shows the model; `display_with_flex.stl` is an additional mesh export. The editable generator is `../build_display.py` (CadQuery 2.8).

## Coordinates

Origin: centre of the cover lens front surface. X right, Y up when viewing the screen; +Z toward the viewer. Flex exits the bottom and extends to the left in front view, as in the supplied front drawing. Flex is flat/unfolded.

## Drawing dimensions used

- Cover lens: 26.50 × 82.45 mm; outer corner radius 1.80 mm.
- Front outer perimeter: R0.50 mm edge fillet, added per user request.
- Viewing aperture: 22.50 × 78.45 mm; corner radius 1.50 mm.
- Lens 1.10 mm, adhesive 0.175 mm, panel 0.63 mm: nominal total 1.905 mm, consistent with the drawing's rounded 1.91 mm.
- Rear panel: 23.81 × 81.05 mm, centred horizontally and aligned with the bottom of the lens. This leaves 1.40 mm at the top. The rear drawing also labels a 1.25 mm top offset and the front side view labels 80.59 mm; these references cannot all define the same panel outline. The explicit rear-view panel size was used for this simplified assembly.

## Estimated details

The rear drawing was scaled using the 82.45 mm lens height over 548 pixels (0.150456 mm/pixel), then mirrored to match front-view coordinates. The FPC outline and component positions were manually traced; small curves are approximated by short straight segments. Expect approximately 0.5–1 mm uncertainty in undimensioned outlines, potentially more in unclear details.

The flex is modelled as a continuous 0.15 mm thick solid at the back of the panel, with a short overlap at the panel connection. Printed traces, solder pads, tiny passives, shielding textures and connector contacts are omitted. PI stiffener blocks use 0.125 mm thickness where indicated. The visible 0.58, 0.975 and 1.225 mm maximum component callouts are used as simplified protrusion envelopes measured from the corresponding flex surface; they are not detailed electronic models. Other component heights are estimated.

The connector marked OK-23CM030-04 is an approximate 7.52 × 2.41 × 1.00 mm block on the rear side of the end pad. Its exact mating geometry and height are unknown. The removable 5 × 5 mm liner tab is omitted. Use this model for assembly visualization and preliminary space planning; confirm actual connector mating and critical clearances against the physical part.

The STEP was re-imported successfully, with 16 valid solid bodies. Validation results are recorded in `validation.json`.

Rebuild in the environment created for this task:

```sh
MPLCONFIGDIR=/tmp/panel-mpl /tmp/panel-cad-venv/bin/python build_display.py
```

Sources: `S870e14d947de436eb41e76d928d5ba2dn.avif` (front/side), `Sed36dbc4f8c34a33ad278f24cc7055dbq.avif` (rear), in the parent directory.
