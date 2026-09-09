# OK-23GF030-04 KiCad library

KiCad symbol, PCB footprint, and simplified STEP model for the 30-pin OCN
`OK-23GF030-04` board-side receptacle. This mates with the
`OK-23GM030-04` plug specified by the Osptek AM319M262928ZS panel drawing.

## Files

- `OK-23GF030-04.kicad_sym` — generic 30-pin connector symbol
- `OK-23GF030-04.pretty/CONN-SMD_30P-OK-23GF030-04.kicad_mod` — 0.4 mm-pitch footprint
- `OK-23GF030-04.3dshapes/CONN-SMD_30P-OK-23GF030-04.step` — simplified STEP model
- `OK-23GF030-04.3dshapes/CONN-SMD_30P-OK-23GF030-04.wrl` — EasyEDA-converted VRML model
- `generate_step.py` — CadQuery source for the STEP model

## Provenance and cautions

The symbol and land pattern were converted from JLCPCB/EasyEDA component
`C9900040133`. The four solder hold-down pads were changed from electrical
pin 0 to unnumbered mechanical pads, and the false pin 0 was removed from
the symbol.

The STEP model is an envelope/placement model aligned to that land pattern.
Its overall envelope is approximately 9.10 x 2.91 x 0.65 mm, including the
solder fittings. It is not manufacturer CAD;
small housing, latch, and mating-interface details are simplified. Confirm
the production footprint, component height, panel engagement, and keep-out
against physical samples or an OCN mechanical drawing before fabrication.

Do not substitute the earlier 24-pin `OK-14F024-04` footprint. It is a
different connector family and pin count.

## KiCad setup

Keep the `ok23_connector` folder at the project root so the footprint's
`${KIPRJMOD}` STEP path resolves. Add `OK-23GF030-04.kicad_sym` through
Preferences -> Manage Symbol Libraries, and add the `.pretty` directory
through Preferences -> Manage Footprint Libraries. Use the nickname
`OK-23GF030-04` for the footprint library.

For JLCPCB assembly, use LCSC/JLCPCB part `C9900040133` and verify current
stock and assembly orientation when ordering.
