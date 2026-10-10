#!/bin/sh
# One conformance file on the hand-written Rust build: child-rpy.sh CACHE NAME
HERE=$(cd "$(dirname "$0")" && pwd)
exec "$HERE/../../rust/target/release/rpy" --echo \
  --mount "$1/test=/t/test" --mount "$HERE/support/test/support=/t/test/support" --mount "$HERE/support/stubs=/t" \
  --path /t --arg "test.$2" "$HERE/driver.py"
