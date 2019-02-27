#!/bin/sh

while true; do
    OVERLAY="./command_web" OVERLAY_WATCH="./command_web/static" cargo run --release -p microwiki web
    echo crashed -- restart in 3 sec
    sleep 3;
done