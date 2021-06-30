use node_metrics::Metrics;

fn main() {
    let metrics = Metrics::new();
    metrics.log()
}
