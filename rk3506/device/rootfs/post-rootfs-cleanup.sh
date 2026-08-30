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

# Do not run the vendor-wide mount scan during sysinit. It probes legacy NAND
# mtdblock devices; mount only the pseudo-filesystems needed by this image.
INITTAB="$TARGET_DIR/etc/inittab"
if [ -f "$INITTAB" ] && grep -q '^::sysinit:/bin/mount -a$' "$INITTAB"; then
    awk '
        $0 == "::sysinit:/bin/mount -a" {
            print "::sysinit:/bin/mount -t devpts devpts /dev/pts"
            print "::sysinit:/bin/mount -t tmpfs tmpfs /dev/shm"
            print "::sysinit:/bin/mount -t sysfs sysfs /sys"
            print "::sysinit:/bin/mount -t configfs configfs /sys/kernel/config"
            print "::sysinit:/bin/mount -t debugfs debugfs /sys/kernel/debug"
            print "::sysinit:/bin/mount -t pstore pstore /sys/fs/pstore"
            print "::sysinit:/bin/mount -t tmpfs tmpfs /tmp"
            print "::sysinit:/bin/mount -t tmpfs tmpfs /run"
            print "::sysinit:/bin/mount -t tmpfs tmpfs /var/log"
            next
        }
        { print }
    ' "$INITTAB" > "$INITTAB.tmp"
    mv "$INITTAB.tmp" "$INITTAB"
fi

# Wi-Fi/BT modules remain available for the Rust application, but no vendor
# hardware initialization or legacy ifup/SSH service should run at boot.
rm -f \
    "$TARGET_DIR/etc/init.d/S36wifibt-init.sh" \
    "$TARGET_DIR/etc/init.d/S40network" \
    "$TARGET_DIR/etc/init.d/S50sshd" \
    "$TARGET_DIR/usr/bin/wifibt-init.sh" \
    "$TARGET_DIR/usr/sbin/sshd" \
    "$TARGET_DIR/usr/bin/ssh" \
    "$TARGET_DIR/usr/bin/scp" \
    "$TARGET_DIR/usr/bin/sftp" \
    "$TARGET_DIR/usr/bin/ssh-keygen" \
    "$TARGET_DIR/usr/bin/ssh-keyscan" \
    "$TARGET_DIR/usr/bin/ssh-add" \
    "$TARGET_DIR/usr/bin/ssh-agent" \
    "$TARGET_DIR/usr/libexec/sftp-server"
rm -rf "$TARGET_DIR/etc/ssh"

# Only the project-managed SD-card mountpoint is used. Remove legacy storage
# names created by the vendor directory setup while preserving /mnt/sdcard.
rm -rf \
    "$TARGET_DIR/mnt/udisk" \
    "$TARGET_DIR/mnt/usb_storage" \
    "$TARGET_DIR/mnt/external_sd"
