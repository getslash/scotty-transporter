#!/bin/bash
set -e

cd /src
cargo build --release

rm -rf /pack
mkdir -p /pack/etc
mkdir -p /pack/usr/bin
cp /src/target/release/transporter /pack/usr/bin/transporter
cp /src/example/transporter.conf /pack/etc
fpm -s dir -t deb -v `grep version Cargo.toml | awk '{print $3}' | sed 's/"//g'` -n transporter -C /pack --config-files etc/transporter.conf -f  .
