#!/usr/bin/env bash
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/.context/google-headless-matrix"
LOGS="$OUT/logs"
RESULTS="$OUT/results.tsv"
DOC="$ROOT/docs/protocol/bowser/google-smoke-results.md"
SLEEP_SECONDS="${BOWSER_GOOGLE_MATRIX_SLEEP_SECONDS:-30}"
ONLY="${BOWSER_GOOGLE_MATRIX_ONLY:-}"
RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)"

mkdir -p "$LOGS"
printf "variant\tstatus\treason\tlog\n" > "$RESULTS"

run_variant() {
  local name="$1"
  shift
  local log="$LOGS/$RUN_ID-$name.log"

  if [ -n "$ONLY" ] && [[ "$name" != *"$ONLY"* ]]; then
    return
  fi

  printf "running %s\n" "$name"
  (
    cd "$ROOT" &&
      env BOWSER_GOOGLE_SMOKE=1 "$@" \
        cargo test -p bowser-cli --test interactive_repl \
        google_search_flow_avoids_captcha -- --ignored --nocapture
  ) >"$log" 2>&1
  local status=$?

  local outcome="pass"
  local reason="completed"
  if [ "$status" -ne 0 ]; then
    outcome="fail"
    if grep -q "/sorry/" "$log"; then
      reason="google_sorry"
    elif grep -q "Our systems have detected unusual traffic" "$log"; then
      reason="unusual_traffic"
    elif grep -q "reCAPTCHA" "$log"; then
      reason="recaptcha"
    elif grep -q "window.location.href.includes('/search')" "$log"; then
      reason="not_search_results"
    elif grep -q "Runtime.disable should be recorded" "$log"; then
      reason="trace_assertion"
    elif grep -q "Runtime.disable should not be recorded" "$log"; then
      reason="trace_assertion"
    elif grep -q "expected Google search URL" "$log"; then
      reason="flow_not_search"
    elif grep -q "ExpectTimeout" "$log"; then
      reason="flow_timeout"
    else
      reason="test_failed"
    fi
  fi

  printf "%s\t%s\t%s\t%s\n" "$name" "$outcome" "$reason" "$log" >> "$RESULTS"
  append_doc_row "$name" "$outcome" "$reason" "$log"
  printf "%s: %s (%s)\n" "$name" "$outcome" "$reason"

  if [ "$SLEEP_SECONDS" != "0" ]; then
    sleep "$SLEEP_SECONDS"
  fi
}

append_doc_row() {
  local name="$1"
  local outcome="$2"
  local reason="$3"
  local log="$4"
  local profile="fresh"
  if [[ "$name" == *"persistent"* ]]; then
    profile="persistent"
  fi
  local note="matrix run $RUN_ID; raw log ${log#"$ROOT"/}"
  local row
  row="| $(date -u +%F) | Bowser matrix | $(escape_md "$name") | $profile | $outcome | $(escape_md "$reason") | $(escape_md "$note") |"
  local tmp
  tmp="$(mktemp)"
  awk -v row="$row" '
    /<!-- google-smoke-results:end -->/ {
      print row
    }
    { print }
  ' "$DOC" > "$tmp" && mv "$tmp" "$DOC"
}

escape_md() {
  local value="$1"
  value="${value//|/\\/}"
  printf "%s" "$value"
}

run_variant "headed-low-runtime-control"
run_variant "addback-launch-headed-native-headless" \
  BOWSER_INTERNAL_STEALTH_FEATURES=launch-headed=off
run_variant "addback-launch-native-window-forced-size" \
  BOWSER_INTERNAL_STEALTH_FEATURES=launch-native-window=off
run_variant "probe-runtime-disable-runtime-events-disabled" \
  BOWSER_INTERNAL_STEALTH_FEATURES=runtime-disable=on
run_variant "probe-accessibility-capture-enabled" \
  BOWSER_INTERNAL_STEALTH_FEATURES=accessibility-capture=on
run_variant "probe-skip-runtime-stability-sampler-skipped" \
  BOWSER_INTERNAL_STEALTH_FEATURES=skip-runtime-stability=on
run_variant "addback-backend-focus-input-js-focus" \
  BOWSER_INTERNAL_STEALTH_FEATURES=backend-focus-input=off
run_variant "addback-enter-submit-pointer-submit" \
  BOWSER_INTERNAL_STEALTH_FEATURES=enter-submit=off
run_variant "addback-backend-pointer-target-js-target" \
  BOWSER_INTERNAL_STEALTH_FEATURES=backend-pointer-target=off
run_variant "headless-low-runtime-persistent-profile" \
  BOWSER_INTERNAL_STEALTH_FEATURES=launch-headed=off \
  BOWSER_GOOGLE_SMOKE_PERSISTENT=1 \
  BOWSER_USER_DATA_DIR="$OUT/persistent-profile"

printf "\nresults written to %s\n" "$RESULTS"
