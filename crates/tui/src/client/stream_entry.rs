//! Shared stream entry seam for Chat Completions / Anthropic Messages / Responses.
//!
//! Scoped consolidation for v0.9.1: wire-protocol adapters stay at the edge
//! (`chat.rs`, `anthropic.rs`, `responses.rs`); this module owns the common
//! open path, HTTP/1.1 fallback policy, and idle-timeout envelope so providers
//! do not re-implement transport differently.
//!
//! Full piagent-style provider collapse is deferred — see
//! `docs/notes/post-0.9.1-thin-tui-and-stream.md`.

use std::time::Duration;

use reqwest::Client;

/// How the shared stream open path should pin HTTP version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamHttpPolicy {
    /// Prefer the dual client (H2 primary, H1 twin for fallback).
    DualWithH1Fallback,
    /// Force HTTP/1.1 only (env pin or prior H2 stall).
    Http1Only,
}

/// Inputs shared by every streaming provider adapter at open time.
#[derive(Debug, Clone)]
pub struct StreamOpenRequest {
    pub policy: StreamHttpPolicy,
    pub open_timeout: Duration,
    pub idle_timeout: Duration,
}

impl StreamOpenRequest {
    #[must_use]
    pub fn new(open_timeout: Duration, idle_timeout: Duration) -> Self {
        Self {
            policy: if super::force_http1_from_env() {
                StreamHttpPolicy::Http1Only
            } else {
                StreamHttpPolicy::DualWithH1Fallback
            },
            open_timeout,
            idle_timeout,
        }
    }

    /// After an H2 stall, retry on the HTTP/1.1 twin.
    #[must_use]
    pub fn with_h1_only(mut self) -> Self {
        self.policy = StreamHttpPolicy::Http1Only;
        self
    }
}

/// Select the HTTP client for a stream open attempt.
#[must_use]
pub fn client_for_policy<'a>(
    primary: &'a Client,
    http1_fallback: &'a Client,
    policy: StreamHttpPolicy,
) -> &'a Client {
    match policy {
        StreamHttpPolicy::DualWithH1Fallback => primary,
        StreamHttpPolicy::Http1Only => http1_fallback,
    }
}

/// Whether a transport error should trigger H1 fallback retry.
#[must_use]
pub fn should_retry_with_h1(policy: StreamHttpPolicy, err_text: &str) -> bool {
    if policy != StreamHttpPolicy::DualWithH1Fallback {
        return false;
    }
    let lower = err_text.to_ascii_lowercase();
    lower.contains("http2")
        || lower.contains("h2 ")
        || lower.contains("stream closed")
        || lower.contains("connection reset")
        || lower.contains("protocol error")
        || lower.contains("frame size")
}

/// Format a stable idle-timeout message shared across adapters.
#[must_use]
pub fn idle_timeout_message(
    idle: Duration,
    bytes_received: usize,
    stream_age: Duration,
    since_last_chunk: Duration,
) -> String {
    format!(
        "SSE stream idle timeout after {}s — no data received \
         (bytes_received={}, stream_age_ms={}, ms_since_last_chunk={})",
        idle.as_secs(),
        bytes_received,
        stream_age.as_millis(),
        since_last_chunk.as_millis(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn h1_retry_only_on_dual_policy() {
        assert!(should_retry_with_h1(
            StreamHttpPolicy::DualWithH1Fallback,
            "http2 protocol error"
        ));
        assert!(!should_retry_with_h1(
            StreamHttpPolicy::Http1Only,
            "http2 protocol error"
        ));
    }

    #[test]
    fn idle_message_is_stable() {
        let msg = idle_timeout_message(
            Duration::from_secs(30),
            0,
            Duration::from_secs(30),
            Duration::from_secs(30),
        );
        assert!(msg.contains("idle timeout"));
        assert!(msg.contains("bytes_received=0"));
    }
}
