#!/usr/bin/env bash
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RESULTS="$ROOT/.context/google-headless-matrix/results.tsv"

bowser_status=0

printf "running Bowser headed low-runtime control\n"
(
  cd "$ROOT" &&
    BOWSER_GOOGLE_MATRIX_ONLY="${BOWSER_GOOGLE_MATRIX_ONLY:-headed-low-runtime-control}" \
    BOWSER_GOOGLE_MATRIX_SLEEP_SECONDS="${BOWSER_GOOGLE_MATRIX_SLEEP_SECONDS:-0}" \
    bash scripts/bowser-google-smoke-matrix.sh
)

if [ -f "$RESULTS" ] && grep -q $'\tfail\t' "$RESULTS"; then
  bowser_status=1
fi

if [ "$bowser_status" -ne 0 ]; then
  printf "\nBowser control failed: %s\n" "$bowser_status"
  exit 1
fi

printf "\nBowser control passed\n"
