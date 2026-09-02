#!/bin/sh

# Remove generic audio support artifacts from packages that are otherwise
# useful on this image. The board has no sound card and no audio application.
rm -f \
    "$TARGET_DIR/usr/lib/udev/rules.d/60-persistent-alsa.rules" \
    "$TARGET_DIR/usr/lib/pm-utils/power.d/intel-audio-powersave" \
    "$TARGET_DIR/usr/share/fish/completions/alsactl.fish" \
    "$TARGET_DIR/usr/share/fish/completions/amixer.fish" \
    "$TARGET_DIR/usr/share/fish/completions/alsamixer.fish" \
    "$TARGET_DIR/usr/share/fish/completions/ffmpeg.fish" \
    "$TARGET_DIR/usr/share/fish/completions/ffplay.fish" \
    "$TARGET_DIR/usr/share/fish/completions/ffprobe.fish" \
    "$TARGET_DIR/usr/share/vim/vim91/syntax/alsaconf.vim" \
    "$TARGET_DIR/usr/share/vim/vim91/ftplugin/alsaconf.vim"

# The SDK may leave files from its default urandom-scripts package in an
# incremental target directory even when Buildroot disables that package.
# Remove the init script and applet link explicitly; this image does not use
# persistent random-seed handling.
rm -f \
    "$TARGET_DIR/etc/init.d/S01seedrng" \
    "$TARGET_DIR/usr/sbin/seedrng" \
    "$TARGET_DIR/usr/bin/seedrng" \
    "$TARGET_DIR/sbin/seedrng"

# Slint's software renderer needs at least one usable font. Keep the board
# image independent of a desktop font installation by shipping Slint's bundled
# Inter font from the vendored dependency tree.
SLINT_FONT=/project/obd2-dashboard-slint/vendor/i-slint-common/sharedfontique/Inter-VariableFont.ttf
if [ -f "$SLINT_FONT" ]; then
    install -d "$TARGET_DIR/usr/share/fonts/slint"
    install -m 0644 "$SLINT_FONT" "$TARGET_DIR/usr/share/fonts/slint/Inter-VariableFont.ttf"
fi

# Overlay copying does not preserve executable bits for these services.
chmod 0755 \
    "$TARGET_DIR/etc/init.d/S00loopback" \
    "$TARGET_DIR/etc/init.d/S15obd2-dashboard"

# Install the direct framebuffer color diagnostic.  It writes pixels according
# to fb0's reported bitfields, so this test is independent of fbcon/vt.color.
FB_TEST_CC="${TARGET_CC:-}"
if [ -z "$FB_TEST_CC" ] && [ -n "${HOST_DIR:-}" ] && \
   [ -x "$HOST_DIR/bin/arm-buildroot-linux-gnueabihf-gcc" ]; then
    FB_TEST_CC="$HOST_DIR/bin/arm-buildroot-linux-gnueabihf-gcc"
fi
if [ -z "$FB_TEST_CC" ] && \
   [ -x /sdk/buildroot/output/rockchip_rk3506_luckfox/host/bin/arm-buildroot-linux-gnueabihf-gcc ]; then
    FB_TEST_CC=/sdk/buildroot/output/rockchip_rk3506_luckfox/host/bin/arm-buildroot-linux-gnueabihf-gcc
fi
if [ -n "$FB_TEST_CC" ]; then
    SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
    FB_TEST_SOURCE=/project/device/rootfs/fb-color-test.c
    [ -f "$FB_TEST_SOURCE" ] || FB_TEST_SOURCE="$SCRIPT_DIR/fb-color-test.c"
    install -d "$TARGET_DIR/usr/bin"
    "$FB_TEST_CC" ${TARGET_CFLAGS:-} ${TARGET_LDFLAGS:-} \
        "$FB_TEST_SOURCE" -o "$TARGET_DIR/usr/bin/fb-color-test"
    chmod 0755 "$TARGET_DIR/usr/bin/fb-color-test"

    FB_RESET_SOURCE=/project/device/rootfs/fb-reset.c
    [ -f "$FB_RESET_SOURCE" ] || FB_RESET_SOURCE="$SCRIPT_DIR/fb-reset.c"
    "$FB_TEST_CC" ${TARGET_CFLAGS:-} ${TARGET_LDFLAGS:-} \
        "$FB_RESET_SOURCE" -o "$TARGET_DIR/usr/bin/fb-reset"
    chmod 0755 "$TARGET_DIR/usr/bin/fb-reset"
fi
