#!/usr/bin/env sh

# rm -rf dist/*
cargo run --release
(cd dist/ && git add -A . && git commit -m "[Auto] Deploy $(date)" && git push -f)
