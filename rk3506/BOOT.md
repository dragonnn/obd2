• Current boot is already quite good:

  - Kernel reaches /sbin/init: ~2.09 s
  - SD card mounted: ~3.16 s
  - ADB gadget configured: ~5.46 s
  - Practical “ready for ADB” point: ~5.5 s

  Main remaining opportunities:

  1. SD-card mount — biggest userspace delay

  EXT4-fs recovery/mount takes roughly 0.55–1.0 s:

  [2.49] mtdblock warnings begin
  [3.15] EXT4 recovery complete
  [3.16] SD mounted

  The SD card is being mounted cleanly enough now, but journal recovery can be slow. Options:

  - Keep ext4 with journaling: safest.
  - Format ext4 without a journal: faster boot, less crash resilience.
  - Do not use noload unless the card is always cleanly unmounted.

  2. Remove the remaining mount -a

  /etc/inittab still runs:

  ::sysinit:/bin/mount -a

  This appears to be responsible for the legacy mtdblock probing:

  MTD device 'rootfs'
  MTD device 'uboot'
  MTD device 'boot'

  Replacing it with only the required pseudo-filesystem mounts should remove the warnings and likely save some boot time. This is the best
  next change.

  3. USB ADB initialization

  ADB becomes configured around 0.65 s after the USB gadget setup starts:

  [4.81] file system registered
  [5.46] USB_STATE=CONFIGURED

  It is already asynchronous, so it does not block the init script. Further improvement would require simplifying the vendor usbdevice
  script/configfs setup. The tradeoff is that ADB becomes available slightly later or with less robust reconnect handling.

  4. Disable unused touchscreen probing

  The kernel spends time probing nonexistent devices:

  - Goodix at 0x5d
  - Goodix at 0x14
  - EDT FT5x06 at 0x38
  - Backlight at 0x45
  - ADC keys

  These produce repeated errors and save perhaps 50–150 ms if removed from the device tree/configuration. The display itself currently
  initializes successfully, so I would remove only unused touchscreen/backlight/ADC nodes first.

  5. Remove FIQ debugger and serial console

  These errors are harmless but unnecessary:

  fiq_debugger ... IRQ fiq not found
  fiq_debugger ... IRQ wakeup not found

  Since interaction is through ADB, disabling the FIQ debugger and removing console=ttyFIQ0 should clean up boot and eliminate the failed
  probe.

  6. Check redundant startup services

  The boot still starts:

  - S36wifibt-init.sh
  - S40network
  - S50sshd
  - S50usbdevice.sh

  Since Wi-Fi/network setup is planned for the Rust application, S36wifibt-init.sh and S40network could likely be removed from the boot-
  critical path. SSH is currently useful, so I would keep it unless ADB fully replaces it.

  Recommended order:

  1. Replace mount -a with explicit required mounts.
  2. Remove unused touchscreen/input device-tree nodes.
  3. Disable FIQ debugger/serial console.
  4. Remove automatic Wi-Fi/network init.
  5. Optimize USB setup only if the ~5.5 s ADB-ready time is still insufficient.

  Implemented:

  - Replaced the boot-time `mount -a` scan with explicit pseudo-filesystem
    mounts, avoiding legacy NAND `mtdblock` probing.
  - Changed the SD-card mount from `LABEL=LYRA_SD` to direct
    `/dev/mmcblk0p1` mounting. After flashing, `dmesg` showed no `mtdblock`
    warnings and the SD card mounted correctly at `/mnt/sdcard`.
  - Removed automatic vendor Wi-Fi/BT initialization and legacy network
    startup; the Rust application will configure wireless devices at runtime.
  - Removed OpenSSH and its init service; ADB, including ADB over TCP, is the
    supported shell/transport. The final boot services include only the
    expected SD-card, logging, async-commit, and USB ADB services.
  - Disabled unused touchscreen, ADC-key, and Waveshare I2C backlight nodes
    and kernel drivers. After flashing, their probe errors were absent while
    the display, SD card, and ADB continued to initialize normally.
  - Disabled the unused FIQ debugger and removed the early UART/FIQ console
    boot arguments. ADB remains the supported console path. After flashing,
    no FIQ/UART probe messages were present and ADB reached configured at
    approximately 4.62 seconds.
  - Tested UBI fastmap with `ubi.fm_autoconvert=1`. The original autoresizing
    rootfs volume occupied all 1787 available PEBs (`available PEBs: 0`), so
    no fastmap could be stored.
  - Tested a fixed 32 MiB static UBI rootfs while retaining the full
    `rootfs:grow` MTD partition. It left 1476 free PEBs, but fastmap still did
    not persist automatically for the read-only rootfs.
  - Switched the immutable rootfs experiment to direct SquashFS on
    `/dev/mtdblock2`, removing UBI attach and fastmap from the boot path.
    The image flashed and booted successfully; the root mounted at about
    1.38 seconds, SD-card mount completed at about 3.06 seconds, and ADB
    reached `CONFIGURED` at about 5.68 seconds. The kernel warns that
    `mtdblock` is being used on NAND; this is the expected tradeoff of the
    direct-MTD experiment and must be weighed against UBI's bad-block safety.
  - Disabled `CONFIG_MTD_UBI` and `CONFIG_MTD_UBI_BLOCK`; direct MTD is now the
    only rootfs storage path. After flashing, no UBI initialization appeared;
    the root mounted at about 1.36 seconds, SD-card mount completed at about
    2.67 seconds, and ADB reached `CONFIGURED` at about 5.30 seconds.
