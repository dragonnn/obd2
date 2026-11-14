#!/bin/sh
# RK3506 GPIO0_B7 only: pulse the reset line without LCD/DRM commands.
# GPIO v2 uses bits 31:16 as write enables for data bits 15:0.
set -eu

hold_seconds=${1:-5}
case "$hold_seconds" in
    ''|*[!0-9]*) echo "Usage: $0 [hold-seconds: 1-60]" >&2; exit 1 ;;
esac
if [ "$hold_seconds" -lt 1 ] || [ "$hold_seconds" -gt 60 ]; then
    echo "Hold time must be between 1 and 60 seconds." >&2
    exit 1
fi

grep -q 'rockchip,rk3506' /proc/device-tree/compatible || exit 1
version=$(devmem 0xff940078 32)
case "$version" in
    0x01000C2B|0x01000c2b|0x0101157C|0x0101157c|0x010219C8|0x010219c8) ;;
    *) echo "Unsupported GPIO register layout: $version" >&2; exit 1 ;;
esac

direction=$(devmem 0xff940008 32)
initial=$(devmem 0xff940000 32)
if [ "$((direction & 0x8000))" -eq 0 ] || [ "$((initial & 0x8000))" -eq 0 ]; then
    echo "GPIO0_B7 must already be an output driven HIGH. No changes made." >&2
    exit 1
fi

release_needed=0
release_reset()
{
    if [ "$release_needed" -eq 1 ]; then
        devmem 0xff940000 32 0x80008000
    fi
}
trap release_reset EXIT
trap 'exit 1' HUP INT TERM

check_level()
{
    data=$(devmem 0xff940000 32)
    pad=$(devmem 0xff940070 32)
    printf 'RST output=%s, pad=%s; GPIO0 data=%s\n' \
        "$(((data >> 15) & 1))" "$(((pad >> 15) & 1))" "$data"
    [ "$(((data >> 15) & 1))" -eq "$1" ]
    [ "$(((pad >> 15) & 1))" -eq "$1" ]
    [ "$((data & 0x7fff))" -eq "$((initial & 0x7fff))" ]
}

printf 'GPIO-only test: holding GPIO0_B7 LOW for %s seconds.\n' "$hold_seconds"
release_needed=1
devmem 0xff940000 32 0x80000000
check_level 0
sleep "$hold_seconds"

printf 'Returning GPIO0_B7 HIGH.\n'
devmem 0xff940000 32 0x80008000
release_needed=0
check_level 1
printf 'Done. Other GPIO0 output latches are unchanged; LCD was not reinitialized.\n'
