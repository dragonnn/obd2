• The main remaining boot-time recommendations were:

  - Make SD-card mounting non-blocking. S00sdcard can currently wait up to 5 seconds; move the mount/log bind into the background if boot must
    continue immediately.

  - Disable unused Wi-Fi/Bluetooth initialization if those radios are not always needed.
  - Disable DHCP on unused eth0/eth1 interfaces.
  - Disable unused services such as irqbalance, input-event-daemon, audio initialization, boot animation, and frecon.
  - Reduce kernel/module scope; many unused drivers are currently built and loaded.
  - Disable USB ACM/gadget functions if only ADB is required.
  - Remove unnecessary Buildroot packages and debug information from release images.
  - Keep the root filesystem as SquashFS; that change already reduces filesystem checks and improves read-only boot behavior.
  - Keep fstrim out of boot; it has already been removed.

  The highest-impact next steps are likely making the SD mount asynchronous and disabling unused Wi-Fi/Bluetooth or network initialization.

  Drop ffmpeg and rest of audio stuff
  Drop host-ntfs-3g
  Drop nanocom
  Drop minicom
  Drop ppd
  Drop python stuff
  Drop curl