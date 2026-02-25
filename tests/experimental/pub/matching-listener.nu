#!/usr/bin/env nuze -X0

use std/assert

zenoh open {scouting: {multicast: {enabled: false}} listen: {endpoints: ["tcp/127.0.0.1:17447"]}} -s "pub"
zenoh open {scouting: {multicast: {enabled: false}} connect: {endpoints: ["tcp/127.0.0.1:17447"]}} -s "sub"
sleep 500ms

let main_id = job id

let _ = job spawn {
    zenoh pub matching-listener -s "pub" test/** | first 2 | job send $main_id
}

# Declare a subscriber to trigger matching=true
let sub_jid = job spawn {
    zenoh sub -s "sub" test/** | first
}

sleep 200ms

# Kill the subscriber to trigger matching=false
job kill $sub_jid

sleep 200ms

let results = job recv --timeout 5sec

assert equal ($results | length) 2
assert equal ($results.0.matching) true
assert equal ($results.1.matching) false
