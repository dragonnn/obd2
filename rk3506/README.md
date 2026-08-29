# Luckfox Lyra Zero W Buildroot environment

This directory provides the Ubuntu 22.04 x86_64 environment required by the
Luckfox Lyra SDK. The SDK itself is not redistributed here.

## Setup

1. Build the container, download the SDK, and materialize its local repositories:

   ```sh
   ./lyra-build image
   ./lyra-build download
   ./lyra-build prepare
   ```

   The download is resumable and currently pins the official
   `Luckfox_Lyra_SDK_250815.tar.gz` archive. To use a newer archive from the
   [Luckfox Drive folder](https://drive.google.com/drive/folders/1OHgViTQN34K7PTiB7fiToUk_arG-vtcP?usp=sharing),
   override both values:

   ```sh
   LYRA_SDK_URL='https://drive.google.com/uc?id=FILE_ID' \
   LYRA_SDK_ARCHIVE='Luckfox_Lyra_SDK_NEW.tar.gz' ./lyra-build download
   ```
2. If `sources.buildroot.net` is inaccessible, install Luckfox's official
   offline Buildroot package cache before building:

   ```sh
   ./lyra-build download-cache
   ```
   The custom CO6300 DTSI/DTS and Buildroot configuration are tracked outside
   the disposable SDK and copied into it automatically before each build.
3. Select the target configuration:

   ```sh
   ./lyra-build lunch
   ```

   Select `4` (**RK3506B_Luckfox_Lyra_Zero_W**), then `0` for SD card or `1`
   for SPI NAND, and finally `0` for Buildroot.
4. Build the complete image:

   ```sh
   ./lyra-build build
   ```

Generated images appear in `sdk/rockdev/` (normally a symlink to
`sdk/output/firmware/`). To configure packages interactively, use
`./lyra-build buildroot-config`; partial targets include `rootfs`, `kernel`,
`uboot`, and `firmware`.

After building, put the board in Rockchip loader mode and flash the generated
image with:

```sh
./lyra-build flash
```

The selected image enables the USB OTG peripheral as a CDC ACM serial gadget.
On the host it appears as `/dev/ttyACM0` (or the next available number); the
board exposes an interactive login on `/dev/ttyGS0` at 115200 baud. The board
default is ADB + ACM; RNDIS is removed from the legacy default configuration.

The SDK tree is bind-mounted into the container, so builds persist on the host
and files are created with the current user's UID/GID. Expect the SDK and build
outputs to consume substantial disk space.

## SD card and fish

The board's removable SD partition is formatted as ext4 with the label
`LYRA_SD`. The firmware mounts it at `/mnt/sdcard` during boot and configures
fish to keep its mutable files under `/mnt/sdcard/fish`:

```text
/mnt/sdcard/fish/config.fish
/mnt/sdcard/fish/fish_history
/mnt/sdcard/fish/completions/
/mnt/sdcard/fish/functions/
/mnt/sdcard/fish/conf.d/
```

Fish 3.6.4 is included in the Buildroot image and ADB starts it by default.
The card must retain the `LYRA_SD` filesystem label for the boot mount hook.

## Rockchip multimedia stack

The image currently disables Rockchip's proprietary `RKADK`/`librockit.so`
stack. This removes its ALSA-linked multimedia runtime, which is unnecessary
on this board because it has no sound card. Basic Linux display support through
DRM/KMS or framebuffer interfaces is unaffected.

If the application later needs Rockchip-specific hardware video overlays,
scaling, rotation, camera-to-display pipelines, or related vendor multimedia
features, revisit this decision and re-enable RKADK/Rockit (and its required
dependencies) instead of relying only on standard Linux display interfaces.
