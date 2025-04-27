#!/bin/bash
gzip -k -f $1
scp -O $1.gz pi@100.92.250.83:/home/pi

base_name=$(basename ${1})

ssh pi@100.92.250.83 -t "gzip -k -f -d /home/pi/$base_name.gz && sudo /home/pi/.cargo/bin/probe-rs run /home/pi/$base_name --chip esp32c6 --disable-progressbars --probe 303a:1001"