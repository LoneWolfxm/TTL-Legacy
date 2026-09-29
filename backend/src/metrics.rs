use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;

/// Shared metrics state for the TTL-Legacy backend.
#[derive(Default)]
pub struct Metrics {
    pub vaults_total: AtomicU64,
    pub checkins_total: AtomicU64,
    pub releases_total: AtomicU64,
    pub active_vaults: AtomicI64,
    pub request_errors_total: AtomicU64,
    pub http_requests_total: AtomicU64,
    pub contract_paused: AtomicU64,
    pub notification_deliveries_total: AtomicU64,
    pub notification_delivery_failures_total: AtomicU64,
    pub cache_hits_total: AtomicU64,
    pub cache_misses_total: AtomicU64,
    pub cache_invalidations_total: AtomicU64,
}

impl Metrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Record a successful notification delivery for the given channel.
    pub fn record_notification_success(&self, channel: &str) {
        self.notification_deliveries_total
            .fetch_add(1, Ordering::Relaxed);
        self.notification_channel_success(channel)
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Record a failed notification delivery for the given channel.
    pub fn record_notification_failure(&self, channel: &str) {
        self.notification_delivery_failures_total
            .fetch_add(1, Ordering::Relaxed);
        self.notification_channel_failure(channel)
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache hit.
    pub fn record_cache_hit(&self) {
        self.cache_hits_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss.
    pub fn record_cache_miss(&self) {
        self.cache_misses_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache entry invalidated by an indexed contract event.
    pub fn record_cache_invalidation(&self) {
        self.cache_invalidations_total
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Per-channel success counter, keyed by channel name (push/email/sms).
    pub fn notification_channel_success(&self, channel: &str) -> &AtomicU64 {
        match channel {
            "email" => &self.notification_email_success_total,
            "sms" => &self.notification_sms_success_total,
            _ => &self.notification_push_success_total,
        }
    }

    /// Per-channel failure counter, keyed by channel name (push/email/sms).
    pub fn notification_channel_failure(&self, channel: &str) -> &AtomicU64 {
        match channel {
            "email" => &self.notification_email_failure_total,
            "sms" => &self.notification_sms_failure_total,
            _ => &self.notification_push_failure_total,
        }
    }

    /// Render all metrics in Prometheus text exposition format.
    pub fn render(&self) -> String {
        let mut out = String::new();

        push_counter(
            &mut out,
            "ttl_legacy_vaults_total",
            "Total vaults created",
            self.vaults_total.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "ttl_legacy_checkins_total",
            "Total check-ins performed",
            self.checkins_total.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "ttl_legacy_releases_total",
            "Total vault releases triggered",
            self.releases_total.load(Ordering::Relaxed),
        );
        push_gauge_i64(
            &mut out,
            "ttl_legacy_active_vaults",
            "Currently active (non-released) vaults",
            self.active_vaults.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "ttl_legacy_request_errors_total",
            "Total API errors",
            self.request_errors_total.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "ttl_legacy_http_requests_total",
            "Total HTTP requests",
            self.http_requests_total.load(Ordering::Relaxed),
        );
        push_gauge(
            &mut out,
            "ttl_legacy_contract_paused",
            "1 if contract is paused, 0 otherwise",
            self.contract_paused.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "ttl_legacy_notification_deliveries_total",
            "Total successful notification deliveries across all channels",
            self.notification_deliveries_total.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "ttl_legacy_notification_delivery_failures_total",
            "Total failed notification deliveries across all channels",
            self.notification_delivery_failures_total.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "ttl_legacy_notification_push_success_total",
            "Total successful push notification deliveries",
            self.notification_push_success_total.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "ttl_legacy_notification_push_failure_total",
            "Total failed push notification deliveries",
            self.notification_push_failure_total.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "ttl_legacy_notification_email_success_total",
            "Total successful email notification deliveries",
            self.notification_email_success_total.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "ttl_legacy_notification_email_failure_total",
            "Total failed email notification deliveries",
            self.notification_email_failure_total.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "ttl_legacy_notification_sms_success_total",
            "Total successful SMS notification deliveries",
            self.notification_sms_success_total.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "ttl_legacy_notification_sms_failure_total",
            "Total failed SMS notification deliveries",
            self.notification_sms_failure_total.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "ttl_legacy_cache_hits_total",
            "Total cache hits",
            self.cache_hits_total.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "ttl_legacy_cache_misses_total",
            "Total cache misses",
            self.cache_misses_total.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "ttl_legacy_cache_invalidations_total",
            "Total cache entries invalidated by indexed contract events",
            self.cache_invalidations_total.load(Ordering::Relaxed),
        );

        out
    }
}

fn push_counter(out: &mut String, name: &str, help: &str, value: u64) {
    out.push_str(&format!("# HELP {name} {help}\n"));
    out.push_str(&format!("# TYPE {name} counter\n"));
    out.push_str(&format!("{name} {value}\n"));
}

fn push_gauge(out: &mut String, name: &str, help: &str, value: u64) {
    out.push_str(&format!("# HELP {name} {help}\n"));
    out.push_str(&format!("# TYPE {name} gauge\n"));
    out.push_str(&format!("{name} {value}\n"));
}

fn push_gauge_i64(out: &mut String, name: &str, help: &str, value: i64) {
    out.push_str(&format!("# HELP {name} {help}\n"));
    out.push_str(&format!("# TYPE {name} gauge\n"));
    out.push_str(&format!("{name} {value}\n"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_contains_all_metrics() {
        let m = Metrics::new();
        m.vaults_total.store(5, Ordering::Relaxed);
        m.checkins_total.store(10, Ordering::Relaxed);
        m.contract_paused.store(1, Ordering::Relaxed);

        let output = m.render();
        assert!(output.contains("ttl_legacy_vaults_total 5"));
        assert!(output.contains("ttl_legacy_checkins_total 10"));
        assert!(output.contains("ttl_legacy_contract_paused 1"));
    }

    #[test]
    fn test_render_prometheus_format() {
        let m = Metrics::new();
        let output = m.render();
        assert!(output.contains("# HELP ttl_legacy_vaults_total"));
        assert!(output.contains("# TYPE ttl_legacy_vaults_total counter"));
        assert!(output.contains("# TYPE ttl_legacy_active_vaults gauge"));
    }

    #[test]
    fn test_notification_channel_metrics_per_channel() {
        let m = Metrics::new();

        m.record_notification_success("push");
        m.record_notification_success("email");
        m.record_notification_success("email");
        m.record_notification_failure("sms");

        let output = m.render();
        assert!(output.contains("ttl_legacy_notification_push_success_total 1"));
        assert!(output.contains("ttl_legacy_notification_email_success_total 2"));
        assert!(output.contains("ttl_legacy_notification_sms_failure_total 1"));
        assert!(output.contains("ttl_legacy_notification_deliveries_total 3"));
        assert!(output.contains("ttl_legacy_notification_delivery_failures_total 1"));
    }

    #[test]
    fn test_notification_channel_counters_are_independent() {
        let m = Metrics::new();

        m.record_notification_success("push");
        m.record_notification_failure("push");

        let output = m.render();
        assert!(output.contains("ttl_legacy_notification_push_success_total 1"));
        assert!(output.contains("ttl_legacy_notification_push_failure_total 1"));
        assert!(output.contains("ttl_legacy_notification_email_success_total 0"));
        assert!(output.contains("ttl_legacy_notification_email_failure_total 0"));
        assert!(output.contains("ttl_legacy_notification_sms_success_total 0"));
        assert!(output.contains("ttl_legacy_notification_sms_failure_total 0"));
    }

    #[test]
    fn test_cache_hit_miss_metrics() {
        let m = Metrics::new();

        m.record_cache_hit();
        m.record_cache_hit();
        m.record_cache_miss();

        let output = m.render();
        assert!(output.contains("ttl_legacy_cache_hits_total 2"));
        assert!(output.contains("ttl_legacy_cache_misses_total 1"));
    }

    #[test]
    fn test_cache_invalidation_after_checkin() {
        let m = Metrics::new();

        // A check-in event is indexed and invalidates the vault's cache entry.
        m.record_cache_invalidation();

        let output = m.render();
        assert!(output.contains("ttl_legacy_cache_invalidations_total 1"));
    }
}
