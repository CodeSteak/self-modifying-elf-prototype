#!/bin/sh

set -e
set -x

rm -Rf out
mkdir -p out

# Create Playload
    cargo build
    mv target/debug/microwiki out/payload

# build out/loader.a
    clang -DSELF_LEN=0 -DPAYLOAD_LEN=0 -o out/loader.a loader/loader.c
	OUT_LEN=$(stat --printf="%s" out/loader.a)
	PAYLOAD_LEN=$(stat --printf="%s" out/payload)
	clang -DSELF_LEN="$OUT_LEN" -DPAYLOAD_LEN="$PAYLOAD_LEN" -o out/loader.a loader/loader.c
	NEW_OUT_LEN=$(stat --printf="%s" out/loader.a)
	[ "$OUT_LEN" -eq "$NEW_OUT_LEN" ]
	cat out/payload >> out/loader.a
	
# build microwiki via loader.a
	OVERLAY="./overlay" out/loader.a export -o out/microwiki
	chmod +x out/microwiki
	
# "Voila"
