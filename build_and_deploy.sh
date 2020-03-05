#!/bin/bash
set -e
DOCKER_HOST="ssh://${USER}@167.172.50.64"
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release
docker-compose -H "$DOCKER_HOST" up --build -d