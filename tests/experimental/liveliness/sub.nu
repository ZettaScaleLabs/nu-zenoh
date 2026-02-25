#!/usr/bin/env nuze -X0

use std/assert

zenoh open -s "1"
let t1 = zenoh liveliness declare-token -s "1" test/1
sleep 100ms

zenoh open -s "2"
let t2 = zenoh liveliness declare-token -s "2" test/2
sleep 100ms

assert equal (zenoh liveliness get test/* -s "1" | get keyexpr | sort) ["test/1" "test/2"]
assert equal (zenoh liveliness get test/* -s "2" | get keyexpr | sort) ["test/1" "test/2"]

zenoh liveliness undeclare-token $t2
sleep 100ms

assert equal (zenoh liveliness get test/* -s "1" | get keyexpr) ["test/1"]

zenoh liveliness undeclare-token $t1
sleep 100ms

assert equal (zenoh liveliness get test/* -s "2" | get keyexpr) []
