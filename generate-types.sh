#!/bin/sh

cd ./src-tauri
cargo run --bin generate-openapi

cd ..
npx tsx openapi/generate-endpoints.ts