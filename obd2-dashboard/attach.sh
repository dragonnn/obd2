#!/usr/bin/env bash

CHIP="esp32c6"
ELF="target/riscv32imac-unknown-none-elf/release/kia_obd2_esp32c3_v2"

while true; do
    env RUST_LOG=error probe-rs reset --chip "$CHIP"

    timeout --signal=INT --kill-after=1s 10s \
        unbuffer env RUST_LOG=error probe-rs attach \
        --chip "$CHIP" "$ELF" \
        --always-print-stacktrace --catch-hardfault \
        | grep -v --line-buffered 'Error: No connected probes were found' \
        | tee -a attach.log

    sleep 0.1
done
