#!/bin/sh

while true; do
    OVERLAY="./overlay/util/web" OVERLAY_WATCH="./overlay/util/web/static" cargo run web
    echo crashed -- restart in 10 sec
    sleep 3;
done