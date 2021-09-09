use prometheus::{
    default_registry, proto::MetricFamily, register_histogram_vec, register_int_counter,
    HistogramOpts, HistogramVec, IntCounter,
};

/// Metrics exposed on /metrics
#[derive(Clone)]
pub struct Metrics {
    pub handled_events: IntCounter,
    pub reconcile_duration: HistogramVec,
}
impl Metrics {
    pub fn new() -> Self {
        let reconcile_histogram = register_histogram_vec!(
            "foo_controller_reconcile_duration_seconds",
            "The duration of reconcile to complete in seconds",
            &[],
            vec![0.01, 0.1, 0.25, 0.5, 1., 5., 15., 60.]
        )
        .unwrap();

        Metrics {
            handled_events: register_int_counter!(
                "foo_controller_handled_events",
                "handled events"
            )
            .unwrap(),
            reconcile_duration: reconcile_histogram,
        }
    }
}
