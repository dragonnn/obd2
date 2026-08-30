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
