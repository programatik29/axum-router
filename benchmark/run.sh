#!/bin/bash
trap 'kill $(jobs -p)' EXIT
cargo build --release --example route_and_layer
echo "" > results
cargo run -q --release --example route_and_layer &
(
echo "No Routing:"; rewrk -t 12 -c 500 -d 10s -h http://localhost:3000
sleep 3
echo "Light Routing:"; rewrk -t 12 -c 500 -d 10s -h http://localhost:3001
sleep 3
echo "Heavy Routing:"; rewrk -t 12 -c 500 -d 10s -h http://localhost:3002
sleep 3
echo "Light Routing Middleware:"; rewrk -t 12 -c 500 -d 10s -h http://localhost:3003
sleep 3
echo "Heavy Routing Middleware:"; rewrk -t 12 -c 500 -d 10s -h http://localhost:3004
)\
| tee -a results
