#!/bin/bash
set -e

pushd $(dirname ${BASH_SOURCE[0]})

for d in */Cargo.toml ; do
    d=$(dirname "$d");
    echo "Generating $d";
    (cd $d && cargo run -p cargo-witgen -- g generate)
done

popd
