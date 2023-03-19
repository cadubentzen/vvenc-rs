#!/bin/bash
ffmpeg -y -f lavfi -i testsrc2=r=30:s=1280x720 -frames:v 30 -pix_fmt yuv420p input.y4m && \
cargo run -p vvencli -- -i input.y4m -o output.vvc && \
vvdecapp -b output.vvc -o output.y4m && \
ffplay output.y4m