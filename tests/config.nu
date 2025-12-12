#!/usr/bin/env nuze -0

use std/assert

zenoh open {mode:"router"}

assert equal (zenoh config | get mode) "router"
