#!/usr/bin/env bash
# scripts/check-wasm-artifact.sh — Regression check for issue #43.
#
# Verifies that the canonical milestonex_campaign.wasm binary is actually
# produced by `make build-wasm` and stays under the 64 KiB Soroban ceiling
# for hosted contracts. Guards against `cargo build` silently succeeding
# while only producing the legacy milestonex_core.wasm (e.g. if a future
# change drops `-p milestonex-campaign` from the Makefile's build-wasm
# target).
#
# Usage:
#   bash scripts/check-wasm-artifact.sh
#
# Run after `make build-wasm` (raw build) or `make optimize` (wasm-opt'd
# build — the artifact deploy tooling actually ships).

set -euo pipefail

WASM_PATH="target/wasm32v1-none/release/milestonex_campaign.wasm"
SIZE_CAP_BYTES=65536

if [ ! -f "$WASM_PATH" ]; then
  echo "❌ $WASM_PATH not found."
  echo "   Expected the canonical campaign contract to be produced by 'make build-wasm'."
  echo "   Run 'make build-wasm' first, or check that milestonex-campaign is still"
  echo "   listed in the Makefile's build-wasm target and its crate-type includes cdylib."
  exit 1
fi

SIZE_BYTES=$(wc -c < "$WASM_PATH")

if [ "$SIZE_BYTES" -ge "$SIZE_CAP_BYTES" ]; then
  echo "❌ $WASM_PATH is ${SIZE_BYTES}B, which is >= the ${SIZE_CAP_BYTES}B (64 KiB)"
  echo "   Soroban ceiling for hosted contracts. Run 'make optimize' (wasm-opt -Oz)"
  echo "   before deploying, or investigate the size regression."
  exit 1
fi

echo "✅ $WASM_PATH exists and is ${SIZE_BYTES}B (< ${SIZE_CAP_BYTES}B / 64 KiB)"
