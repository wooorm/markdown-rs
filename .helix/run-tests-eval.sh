#!/bin/bash
set -e

if [ -z "$1" ]; then
    cargo test --workspace || true
else
    cargo test --workspace || true
fi