#!/bin/bash
gzip -f $1
scp -O $1.gz pi@X.X.X.X:/home/pi

base_name=$(basename ${1})

ssh pi@X.X.X.X -t "gzip -f -d /home/pi/$base_name.gz && sudo /home/pi/.cargo/bin/probe-rs run /home/pi/$base_name --chip esp32c6"