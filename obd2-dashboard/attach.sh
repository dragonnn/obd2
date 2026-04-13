#!/bin/bash

while true; do 
    unbuffer env RUST_LOG=error probe-rs attach --chip esp32c6 target/riscv32imac-unknown-none-elf/release/kia_obd2_esp32c3_v2 | grep -v --line-buffered 'Error: No connected probes were found' | tee -a attach.log
    sleep 0.1
done