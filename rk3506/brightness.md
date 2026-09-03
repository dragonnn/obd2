 I read the complete converted datasheet. The important conclusion is:

  The CO6300 does not use PWM for panel brightness. Brightness is controlled through MIPI DCS commands, primarily 0x51.

  amoled/3.19寸CO6300-MIPI资料/CO_6300_Datasheet_V0_00_20240827_for_customer_d6da76ca4d.md

  ## Normal brightness

  ### 0x51 — WRDISBV

  Write Display Brightness:

  51h, DBV[7:0]

  - 0x00 = minimum brightness
  - 0xFF = maximum brightness
  - 0x01 ≈ 0.78%
  - 0xFE ≈ 99.6%
  - Power-on/reset default = 0x00

  This is explicitly valid while the panel is in:

  - Normal display mode
  - Idle mode
  - Partial mode
  - Sleep-out state
  - The datasheet also lists Sleep In as register-accessible

  For Linux, this should be sent as a one-parameter DCS write:

  MIPI DSI data type: 0x15
  DCS command:       0x51
  Parameter:         brightness byte

  Our use of:

  mipi_dsi_dcs_write(dsi, 0x51, &brightness, 1);

  is therefore the correct MIPI packet format.

  The relevant section is around amoled/3.19寸CO6300-MIPI资料/CO_6300_Datasheet_V0_00_20240827_for_customer_d6da76ca4d.md:10046.

  ### 0x52 — RDDISBV

  Reads back the normal brightness register:

  52h -> DBV[7:0]

  This would be useful for verifying that the panel accepted our runtime writes.

  ## Brightness-control gate and dimming

  ### 0x53 — WRCTRLD

  This controls whether 0x51 is actually used:

   Bit    Name             Meaning
  ━━━━━  ━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   D5     BCTRL / BC_EN    Enables the normal brightness control block
  ─────  ───────────────  ─────────────────────────────────────────────
   D3     DD / DIM_EN      Enables display dimming

  Values:

  0x00: brightness control disabled, dimming disabled
  0x20: brightness control enabled, dimming disabled
  0x08: brightness control disabled, dimming enabled
  0x28: brightness control enabled, dimming enabled

  The datasheet’s documented default is 0x28, but its reset value is shown as 0x00. That is an inconsistency.

  The vendor initialization examples use both patterns:

  0x53, 0x20

  in the ESP-IDF initialization code, while the datasheet default table reports 0x28.

  The important practical difference is:

  - 0x20 enables direct DBV brightness control without dimming.
  - 0x28 enables DBV brightness control with the dimming function active.

  Given that 0x28 produced a black panel in our testing while 0x20 was known to initialize correctly, 0x20 is the safer value for direct brightness
  testing.

  ### 0x54 — RDCTRLD

  Reads back the brightness-control state, including BCTRL and DD. This should be added to diagnostics if the DSI host supports panel reads.

  ## HBM brightness

  ### 0x63 — WRHBMDISBV

  This is not the ordinary brightness register.

  63h, DBV_HBM[7:0]

  It stores the brightness value used while HBM mode is active:

  - 0x00 = minimum HBM brightness
  - 0xFF = maximum HBM brightness
  - Reset default = 0x00

  Writing 0x63 alone should not control normal display brightness. It only matters after HBM has been enabled.

  The datasheet section is around amoled/3.19寸CO6300-MIPI资料/CO_6300_Datasheet_V0_00_20240827_for_customer_d6da76ca4d.md:10975.

  ### 0x66 — HBM enable

  This switches HBM mode:

  66h, 00h  -> HBM disabled
  66h, 01h  -> HBM enabled

  The datasheet says this enters or exits HBM mode, but the section contains a typo and calls the command “read” even though it is documented as a
  write.

  Therefore, an HBM sequence would theoretically be:

  53h, 20h       // normal DBV control enabled
  51h, xx        // normal brightness
  63h, yy        // HBM brightness
  66h, 01h       // enter HBM

  However, the vendor initialization code writes 0x63 0xFF but does not write 0x66 0x01, so it is only preloading the HBM brightness register.

  ## ACL and sunlight enhancement

  ### 0x55 — ACL control

  The datasheet says 0x55 controls Auto Current Limit:

  55h, 00h       // ACL disabled
  55h, 03h       // ACL enabled

  ACL is intended to reduce AMOLED power consumption and extend panel life. It can also reduce brightness under high-area illumination, so it should
  remain disabled while testing brightness.

  There is an inconsistency: some command tables label 0x55 as CABC, while the detailed command section identifies it as ACL. The detailed ACL
  section is the more useful reference.

  ### 0x56

  Reads the ACL state.

  ### 0x58 — SRE / sunlight-readable enhancement

  Despite some vendor comments calling this a brightness command, the datasheet describes 0x58 as sunlight-readable enhancement:

  SLR_EN
  SLR_LEVEL[1:0]

  - SLR enable/disable
  - Low, medium, or high enhancement level

  It is not the primary brightness register. The vendor’s 0x58 0x07 enables enhancement at a configured level, but it should not replace 0x51.

  ### 0x5F

  Controls local HBM:

  0x00: local HBM disabled
  0x01: local HBM enabled

  This is separate from global HBM brightness.

  ## MIPI packet requirements

  The datasheet explicitly defines these packet types:

   Operation                             DSI packet type
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━
   DCS write, no parameter               0x05
  ────────────────────────────────────  ─────────────────
   DCS write, one parameter              0x15
  ────────────────────────────────────  ─────────────────
   DCS long write                        0x39
  ────────────────────────────────────  ─────────────────
   Generic short write, one parameter    0x13
  ────────────────────────────────────  ─────────────────
   Generic long write                    0x29

  For brightness:

  0x51 xx -> DCS short write, one parameter, packet type 0x15
  0x53 xx -> DCS short write, one parameter, packet type 0x15
  0x63 xx -> DCS short write, one parameter, packet type 0x15
  0x66 xx -> DCS short write, one parameter, packet type 0x15

  The datasheet shows DCS writes being transmitted either through:

  - LPDT — Low-Power Data Transmission
  - HSDT — High-Speed Data Transmission

  For a one-parameter write, the normal sequence is:

  LP-11
  DCS write packet
  LP-11

  or the equivalent HSDT sequence with EoTP.

  The Linux DSI controller handles that transport detail. The panel driver should only use the correct DCS API.

  ## Vendor initialization sequence

  The vendor MIPI example contains this relevant sequence:

  53 20
  51 FF
  63 FF
  11
  delay 60/120 ms
  29
  51 FF
  58 07

  The ESP-IDF source also includes a second 0x11, resulting in:

  51 FF
  11
  delay
  63 FF
  29
  51 FF
  58 07

  The duplicate 0x51 is probably harmless initialization redundancy. It does not imply that brightness must be written twice at runtime.

  ## What this means for our driver

  The correct minimal runtime design is:

  Initialization:
      53 20
      51 FF
      63 FF       // optional HBM preload only
      11
      delay
      29

  Runtime brightness:
      51 brightness

  We should not use the PWM backlight node for the AMOLED brightness path. The panel has no documented PWM brightness interface.

  The most suspicious parts of our current implementation are:

  1. 0x63 was treated as if it were normal brightness, although it is HBM-only.
  2. 0x28 enables dimming and caused the panel failure during testing; 0x20 is safer.
  3. The current driver writes 0x51 during initialization and again from prepare(). The second write is unnecessary.
  4. The prepare() failure path is not what caused the black screen in the logged test: the callback returned 0, so mipi_dsi_dcs_set_display_off()
     was not executed.

  5. The datasheet confirms that 0x51 is the correct command and that the DSI packet type must be DCS one-parameter write (0x15), which our
     mipi_dsi_dcs_write() call generates.

  The next clean test should therefore keep:

  0x53 = 0x20

  remove runtime writes to 0x63, leave PWM disabled as a brightness provider, and expose only native DCS 0x51 control.