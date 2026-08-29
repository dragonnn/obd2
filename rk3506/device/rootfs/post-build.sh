#!/bin/sh

# Remove generic audio support artifacts from packages that are otherwise
# useful on this image. The board has no sound card and no audio application.
rm -f \
    "$TARGET_DIR/usr/lib/udev/rules.d/60-persistent-alsa.rules" \
    "$TARGET_DIR/usr/lib/pm-utils/power.d/intel-audio-powersave" \
    "$TARGET_DIR/usr/lib/python3.11/lib-dynload/audioop.cpython-311-arm-linux-gnueabihf.so" \
    "$TARGET_DIR/usr/lib/python3.11/email/mime/audio.pyc" \
    "$TARGET_DIR/usr/share/fish/completions/alsactl.fish" \
    "$TARGET_DIR/usr/share/fish/completions/amixer.fish" \
    "$TARGET_DIR/usr/share/fish/completions/alsamixer.fish" \
    "$TARGET_DIR/usr/share/vim/vim91/syntax/alsaconf.vim" \
    "$TARGET_DIR/usr/share/vim/vim91/ftplugin/alsaconf.vim"
