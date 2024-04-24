#!/bin/bash
probe-rs download --chip esp32c6 --connect-under-reset $1
probe-rs reset --chip esp32c6 

#socat tcp-l:4444,reuseaddr,fork file:/dev/ttyACM0,nonblock &

#defmt-print -e $1 tcp --host 127.0.0.1 --port 4444

trap ctrl_c INT

function ctrl_c() {
    echo "Ctrl+C pressed"
    killall -9 socat
    killall -9 defmt-print
    killall -9 tio
    killall -9 screen
    exit 0
}



#socat /dev/ttyACM0,b115200,raw,echo=0,crnl - | defmt-print --show-skipped-frames -e $1

#unbuffer cat /dev/ttyACM0 | defmt-print --show-skipped-frames -e $1

screen -d -m tio --mute /dev/ttyACM0 -S inet:4444

defmt-print -v --show-skipped-frames -e $1 tcp --port 4444

killall -9 tio
killall -9 screen