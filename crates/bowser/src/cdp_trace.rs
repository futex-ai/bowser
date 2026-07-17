//! Optional CDP method tracing without command parameters.

use std::env;
use std::fs::OpenOptions;
use std::io::Write;

use serde::Serialize;

const TRACE_PATH_ENV: &str = "BOWSER_CDP_TRACE_PATH";

#[derive(Serialize)]
struct CdpTraceEntry<'a> {
    timestamp: String,
    method: &'a str,
    high_risk_google_method: bool,
}

pub(crate) fn record_method(method: &str) {
    let Some(path) = env::var_os(TRACE_PATH_ENV) else {
        return;
    };
    let entry = CdpTraceEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        method,
        high_risk_google_method: is_high_risk_google_method(method),
    };
    let Ok(line) = serde_json::to_string(&entry) else {
        return;
    };
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = writeln!(file, "{line}");
}

pub(crate) fn is_high_risk_google_method(method: &str) -> bool {
    method == "Runtime.enable"
}

#[cfg(test)]
#[path = "_tests_/cdp_trace_tests.rs"]
mod cdp_trace_tests;
