#!/bin/bash
set -e
docker_host="ssh://${USER}@167.172.50.64"
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release
docker-compose -H "$docker_host" up --build -d