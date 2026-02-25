#!/usr/bin/env nuze -0

use std/assert

zenoh open {id: "abc123" scouting: {multicast: {enabled: false}} listen: {endpoints: []}}

assert equal (zenoh info) {zid: "abc123" routers_zid: [] peers_zid: [] locators: []}
