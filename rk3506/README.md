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
