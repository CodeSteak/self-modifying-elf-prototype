#!/bin/sh

while true; do
    OVERLAY="./overlay/util/web" OVERLAY_WATCH="./overlay/util/web/static" cargo run --release web
    echo crashed -- restart in 10 sec
    sleep 10;
done