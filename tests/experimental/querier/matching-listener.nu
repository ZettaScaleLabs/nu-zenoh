#!/usr/bin/env nuze -X0

use std/assert

zenoh open {scouting: {multicast: {enabled: false}} listen: {endpoints: ["tcp/127.0.0.1:17448"]}} -s "querier"
zenoh open {scouting: {multicast: {enabled: false}} connect: {endpoints: ["tcp/127.0.0.1:17448"]}} -s "queryable"
sleep 500ms

let main_id = job id

let _ = job spawn {
    zenoh querier matching-listener -s "querier" test/** | first 2 | job send $main_id
}

# Declare a queryable to trigger matching=true
let queryable_jid = job spawn {
    zenoh queryable -s "queryable" test/** {|q| [] } | first
}

sleep 200ms

# Kill the queryable to trigger matching=false
job kill $queryable_jid

sleep 200ms

let results = job recv --timeout 5sec

assert equal ($results | length) 2
assert equal ($results.0.matching) true
assert equal ($results.1.matching) false
