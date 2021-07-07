use node_metrics::MetricsManager;

fn main() {
    let mut metrics_manager = MetricsManager::new();
    metrics_manager.log();
    let metrics = metrics_manager.fetch();
    let json = metrics.to_json().unwrap();
    println!("{}", json);
}
