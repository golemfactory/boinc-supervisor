#!/bin/bash

set -e

cargo build --release --target x86_64-unknown-linux-musl
cp ../target/x86_64-unknown-linux-musl/release/boinc-supervisor ./
docker build . -t boinc-supervisor
