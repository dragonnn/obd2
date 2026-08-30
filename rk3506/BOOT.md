
• The full log shows:

          Time    Event
  ━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
        0.87 s    Kernel starts CPUs
  ─────────────  ──────────────────────────────────────────────
        2.09 s    Kernel launches /sbin/init
  ─────────────  ──────────────────────────────────────────────
   2.75–2.88 s    Unnecessary NAND mtdblock warnings/probing
  ─────────────  ──────────────────────────────────────────────
        3.26 s    SD card EXT4 mount completes
  ─────────────  ──────────────────────────────────────────────
        6.80 s    Kernel random generator becomes ready
  ─────────────  ──────────────────────────────────────────────
        7.83 s    USB FunctionFS registers
  ─────────────  ──────────────────────────────────────────────
        8.58 s    USB gadget reaches CONFIGURED; ADB is usable

  So the practical boot endpoint is currently about 8.6 seconds, when ADB becomes available. There is no explicit userspace “boot complete”
  marker, so the exact rcS completion time is not visible.

  The largest remaining delay is likely S01seedrng. It runs immediately after the SD mount and appears to wait until the kernel CRNG
  becomes ready. That accounts for most of the gap between 3.26 s and 6.80 s.

  Recommended improvements, in order:
     limited.

  2. Remove legacy NAND partition probing from mount-helper; the mtdblock warnings for rootfs, uboot, and boot are unnecessary.
  The EXT4 mount itself takes only about 0.01 s according to the timestamps:

  3.259812  recovery complete
  3.266386  filesystem mounted

  The apparent multi-second delay is therefore caused by later early-userspace work, primarily entropy initialization/seedrng, not EXT4.