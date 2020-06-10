#!/bin/sh

slot="$1"
shift

test -d "$slot" || { echo "slot dir not found"; exit 1; }

cd "$slot"
/boinc-supervisor &
$@
