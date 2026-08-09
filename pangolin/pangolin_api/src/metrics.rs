//! Prometheus metrics.
//!
//! There were none: no way to answer "what is p99 commit latency", "what is the
//! table-commit conflict rate", or "how many 5xx in the last five minutes"
//! (A-18/C-21). Every monitoring, SLO and alerting requirement depends on this
//! existing first.
//!
//! The exposition format is written by hand rather than pulling in a client
//! library, to keep the dependency surface of a security release small. The
//! output is standard Prometheus text format version 0.0.4.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use axum::http::header;
use axum::response::IntoResponse;

/// Upper bounds, in seconds, of the request-duration histogram.
const LATENCY_BUCKETS: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

#[derive(Default)]
struct RequestSeries {
    count: u64,
    sum_seconds: f64,
    buckets: Vec<u64>,
}

#[derive(Default)]
struct Registry {
    /// (method, route, status class) -> latency series
    requests: Mutex<HashMap<(String, String, u16), RequestSeries>>,
}

fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    REGISTRY.get_or_init(Registry::default)
}

macro_rules! counters {
    ($($vis:vis $name:ident => $metric:literal, $help:literal;)*) => {
        $(
            $vis static $name: AtomicU64 = AtomicU64::new(0);
        )*
        fn render_counters(out: &mut String) {
            $(
                out.push_str(&format!("# HELP {} {}\n", $metric, $help));
                out.push_str(&format!("# TYPE {} counter\n", $metric));
                out.push_str(&format!("{} {}\n", $metric, $name.load(Ordering::Relaxed)));
            )*
        }
    };
}

counters! {
    pub COMMITS_TOTAL            => "pangolin_table_commits_total",            "Table commits attempted.";
    pub COMMITS_SUCCEEDED        => "pangolin_table_commits_succeeded_total",  "Table commits that were published.";
    pub COMMITS_CONFLICTED       => "pangolin_table_commits_conflicted_total", "Table commits rejected because a requirement did not hold.";
    pub COMMIT_CAS_RETRIES       => "pangolin_table_commit_cas_retries_total", "Compare-and-swap retries during table commits.";
    pub AUTH_SUCCESS             => "pangolin_auth_success_total",             "Requests that authenticated successfully.";
    pub AUTH_FAILURE             => "pangolin_auth_failure_total",             "Requests rejected by authentication.";
    pub REVOCATION_CHECK_ERRORS  => "pangolin_token_revocation_check_errors_total", "Token revocation checks that could not be completed.";
    pub AUDIT_WRITE_FAILURES     => "pangolin_audit_write_failures_total",     "Audit records that could not be persisted.";
    pub WAREHOUSE_CACHE_HITS     => "pangolin_warehouse_cache_hits_total",     "Warehouse lookups served from the in-process cache.";
    pub WAREHOUSE_CACHE_MISSES   => "pangolin_warehouse_cache_misses_total",   "Warehouse lookups that reached the store.";
}

/// Increment a counter by one.
pub fn inc(counter: &'static AtomicU64) {
    counter.fetch_add(1, Ordering::Relaxed);
}

/// Record one completed HTTP request.
pub fn record_request(method: &str, route: &str, status: u16, seconds: f64) {
    let mut guard = registry()
        .requests
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let series = guard
        .entry((method.to_string(), route.to_string(), status))
        .or_insert_with(|| RequestSeries {
            count: 0,
            sum_seconds: 0.0,
            buckets: vec![0; LATENCY_BUCKETS.len()],
        });
    series.count += 1;
    series.sum_seconds += seconds;
    for (i, bound) in LATENCY_BUCKETS.iter().enumerate() {
        if seconds <= *bound {
            series.buckets[i] += 1;
        }
    }
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Render the current metrics in Prometheus text exposition format.
pub fn render() -> String {
    let mut out = String::with_capacity(4096);

    out.push_str("# HELP pangolin_http_requests_total HTTP requests handled.\n");
    out.push_str("# TYPE pangolin_http_requests_total counter\n");
    let guard = registry()
        .requests
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    for ((method, route, status), series) in guard.iter() {
        out.push_str(&format!(
            "pangolin_http_requests_total{{method=\"{}\",route=\"{}\",status=\"{}\"}} {}\n",
            escape(method),
            escape(route),
            status,
            series.count
        ));
    }

    out.push_str("# HELP pangolin_http_request_duration_seconds Request latency.\n");
    out.push_str("# TYPE pangolin_http_request_duration_seconds histogram\n");
    for ((method, route, status), series) in guard.iter() {
        let labels = format!(
            "method=\"{}\",route=\"{}\",status=\"{}\"",
            escape(method),
            escape(route),
            status
        );
        for (i, bound) in LATENCY_BUCKETS.iter().enumerate() {
            out.push_str(&format!(
                "pangolin_http_request_duration_seconds_bucket{{{labels},le=\"{bound}\"}} {}\n",
                series.buckets[i]
            ));
        }
        out.push_str(&format!(
            "pangolin_http_request_duration_seconds_bucket{{{labels},le=\"+Inf\"}} {}\n",
            series.count
        ));
        out.push_str(&format!(
            "pangolin_http_request_duration_seconds_sum{{{labels}}} {}\n",
            series.sum_seconds
        ));
        out.push_str(&format!(
            "pangolin_http_request_duration_seconds_count{{{labels}}} {}\n",
            series.count
        ));
    }
    drop(guard);

    render_counters(&mut out);

    out.push_str("# HELP pangolin_ready Whether this instance reports itself ready.\n");
    out.push_str("# TYPE pangolin_ready gauge\n");
    out.push_str(&format!(
        "pangolin_ready {}\n",
        u8::from(crate::health::is_ready())
    ));

    out
}

/// `GET /metrics`
pub async fn metrics_handler() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        render(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_are_counted_and_bucketed() {
        record_request("GET", "/test-metrics-route", 200, 0.02);
        record_request("GET", "/test-metrics-route", 200, 3.0);
        let out = render();
        assert!(out.contains(
            "pangolin_http_requests_total{method=\"GET\",route=\"/test-metrics-route\",status=\"200\"} 2"
        ));
        // 0.02s falls in the 0.025 bucket, 3.0s does not.
        assert!(out.contains("route=\"/test-metrics-route\",status=\"200\",le=\"0.025\"} 1"));
        assert!(out.contains("route=\"/test-metrics-route\",status=\"200\",le=\"+Inf\"} 2"));
    }

    #[test]
    fn counters_render_in_exposition_format() {
        inc(&COMMITS_TOTAL);
        let out = render();
        assert!(out.contains("# TYPE pangolin_table_commits_total counter"));
        assert!(out.contains("# HELP pangolin_auth_failure_total"));
        assert!(out.contains("pangolin_ready "));
    }

    #[test]
    fn label_values_are_escaped() {
        record_request("GET", "/weird\"route", 500, 0.001);
        let out = render();
        assert!(out.contains("route=\"/weird\\\"route\""));
    }
}
