#!/bin/bash
cd `dirname $0`
ARTIFACTS_DIR=artifacts
rm -rf $ARTIFACTS_DIR
mkdir $ARTIFACTS_DIR

for CONTRACT_DIR in contracts/"$@"; do
    export RUSTFLAGS='--emit=llvm-ir -C instrument-coverage -Zno-profiler-runtime'
    export CARGO_TARGET_DIR=build-cov
    cargo +nightly build --release --lib --target=wasm32-unknown-unknown
    for WASM in $CARGO_TARGET_DIR/wasm32-unknown-unknown/release/*.wasm; do
        NAME=$(basename "$WASM" .wasm)
        LLFILE=$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/deps/$NAME.ll
        clang -o $ARTIFACTS_DIR/$NAME.o -c $LLFILE
        cp $WASM $ARTIFACTS_DIR
    done
done
