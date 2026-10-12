#!/usr/bin/env bash
# Run from the SDK directory through lyra-build.
set -euo pipefail

boot_sources=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
install -D "$boot_sources/rk3506b_lyra_can.config" u-boot/configs/rk3506b_lyra_can.config
install -D "$boot_sources/RK3506BMINIALL-no-uart.ini" rkbin/RKBOOT/RK3506BMINIALL-no-uart.ini

ddr_source=rkbin/bin/rk35/rk3506b_ddr_750MHz_v1.04.bin
ddr_output=rkbin/bin/rk35/rk3506b_ddr_750MHz_v1.04-no-uart.bin
ddr_work=$(mktemp -d)
trap 'rm -rf "$ddr_work"' EXIT

# The vendor tool resets omitted parameters. Keep its complete template with
# blank values for everything except the two UART settings below.
sed -e 's/^uart id=.*/uart id=0xf/' \
    -e 's/^dis_train_print=.*/dis_train_print=1/' \
    rkbin/tools/ddrbin_param.txt > "$ddr_work/params.txt"
install -m 0644 "$ddr_source" "$ddr_output"
rkbin/tools/ddrbin_tool rk3506 -g "$ddr_work/before.txt" "$ddr_source" > "$ddr_work/read.log"
if ! rkbin/tools/ddrbin_tool rk3506 "$ddr_work/params.txt" "$ddr_output" > "$ddr_work/modify.log" 2>&1; then
    cat "$ddr_work/modify.log" >&2
    exit 1
fi
rkbin/tools/ddrbin_tool rk3506 -g "$ddr_work/after.txt" "$ddr_output" > "$ddr_work/read.log"

python3 - "$ddr_work/before.txt" "$ddr_work/after.txt" <<'PY'
import sys
from pathlib import Path

def read_config(path):
    return dict(line.split('=', 1) for line in Path(path).read_text().splitlines()
                if '=' in line and not line.startswith('/*'))

before, after = map(read_config, sys.argv[1:])
expected = dict(before, **{'uart id': '15', 'dis_train_print': '1'})
if after != expected:
    raise SystemExit('DDR UART patch changed unexpected settings or did not disable UART0')
print('DDR loader UART disabled; all other DDR settings preserved.')
PY
