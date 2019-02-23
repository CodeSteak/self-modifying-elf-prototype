#/bin/sh

set -e
set -x

rm -Rf out
mkdir -p out

# Create Playload
    cargo build --release
    mv target/release/microwiki out/payload
    strip out/payload
    upx --best out/payload

# build out/loader.a
    clang -DSELF_LEN=0 -DPAYLOAD_LEN=0 -o out/loader.a loader/loader.c
	strip out/loader.a
	OUT_LEN=$(stat --printf="%s" out/loader.a)
	PAYLOAD_LEN=$(stat --printf="%s" out/payload)
	clang -DSELF_LEN="$OUT_LEN" -DPAYLOAD_LEN="$PAYLOAD_LEN" -o out/loader.a loader/loader.c
    strip out/loader.a
	NEW_OUT_LEN=$(stat --printf="%s" out/loader.a)
	[ "$OUT_LEN" -eq "$NEW_OUT_LEN" ]
	cat out/payload >> out/loader.a
	
# build microwiki via loader.a
	OVERLAY="./overlay" USE_UPX=1 USE_STRIP=1 out/loader.a export -o out/microwiki
	chmod +x out/microwiki
	
# "Voila"
