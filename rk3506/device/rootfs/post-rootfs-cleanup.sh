#!/bin/bash

set -e

TARGET_DIR="$1"

# Remove stale files from the vendor post-rootfs target. The SDK can retain
# these even when Buildroot's urandom-scripts package is disabled.
rm -f \
    "$TARGET_DIR/etc/init.d/S01seedrng" \
    "$TARGET_DIR/usr/sbin/seedrng" \
    "$TARGET_DIR/usr/bin/seedrng" \
    "$TARGET_DIR/sbin/seedrng" \
    "$TARGET_DIR/usr/bin/mount-helper" \
    "$TARGET_DIR/usr/bin/disk-helper"

# Mounting is handled by the kernel and the project S00sdcard service. The
# vendor mount-all service also probes legacy NAND partitions, so remove any
# stale copy left by the SDK post-rootfs stages.
find "$TARGET_DIR/etc/init.d" -maxdepth 1 -type f -name '*mountall*.sh' -delete 2>/dev/null || true
