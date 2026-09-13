# Luckfox Lyra Zero W — KiCad library

Contents:
- `Luckfox_Lyra.kicad_sym` — 40-pin symbol with 5V / 3V3 / GND and RMIO/GPIO names.
- `Luckfox_Lyra.pretty/Luckfox_Lyra_Zero_W_40Pin.kicad_mod` — footprint.
- `Luckfox_Lyra.3dshapes/Luckfox_Lyra_Zero_W.step` — simplified generated STEP model.

## Footprint philosophy
The footprint intentionally has **no F.SilkS board drawing**. The only PCB fabrication geometry is the 2×20 through-hole connector. The SBC board outline, four mounting holes, and mechanical notes are on `Cmts.User` so they are visible while laying out the carrier but do not print on silkscreen.

Board envelope: 65.0 × 30.0 mm. Mounting pattern: 58 × 23 mm, four Ø3.0 mm holes.

The 40-pin expansion connector uses 2.54 mm pitch. Pin 1 is the square pad. Pin numbering follows Raspberry-Pi-style alternating numbering: odd pins on the inner row, even pins on the outer/board-edge row.

## 3D model
Luckfox's documentation repository states that hardware resources include a `Hardware/3D Models/` directory, but an exact official Lyra Zero W STEP file was not available through the public indexed resources used while creating this package. Therefore the included STEP is generated from the published 65 × 30 mm board envelope and mounting geometry. It includes approximate major component envelopes and a generic installed 2×20 header for mechanical visualization. **Do not use component positions/heights from this STEP for a tight enclosure without measuring the real board.**

## Electrical pin mapping
RMIO/GPIO labels were transcribed from the official Luckfox Lyra Zero W schematic.

## Mechanical assumptions
The header and mounting-hole coordinates use the Raspberry Pi Zero-compatible 65 × 30 mm / 58 × 23 mm mechanical convention, consistent with Luckfox's stated Pi-HAT compatibility and published outline. If your carrier has very tight registration requirements, verify pin-1-to-board-edge dimensions against a physical Lyra Zero W before production.
