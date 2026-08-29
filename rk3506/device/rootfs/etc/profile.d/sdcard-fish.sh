# Keep fish's mutable user files off the rootfs.
if mount | grep -q ' on /mnt/sdcard '; then
	export XDG_CONFIG_HOME=/mnt/sdcard
	export XDG_DATA_HOME=/mnt/sdcard
	if [ -x /usr/bin/fish ] && [ -t 0 ] && [ -z "$FISH_VERSION" ]; then
		exec /usr/bin/fish
	fi
fi
