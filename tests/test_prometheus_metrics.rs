//! Test Prometheus metrics parsing
//!
//! This test helps debug consensus data issues by fetching and displaying
//! the actual Prometheus metrics from the node.

#[tokio::test]
async fn test_display_prometheus_metrics() {
    use reqwest::Client;

    let client = Client::new();
    let response = match client.get("http://localhost:8889/metrics").send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Cannot connect to Prometheus endpoint: {}", e);
            eprintln!("This test requires a running monad node on localhost:8889");
            return;
        }
    };

    let body = match response.text().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to read response: {}", e);
            return;
        }
    };

    println!("\n=== PROMETHEUS METRICS ===\n");

    // Show all consensus-related metrics
    println!("--- CONSENSUS METRICS ---");
    for line in body.lines() {
        if line.contains("consensus") || line.contains("epoch") || line.contains("round") {
            println!("{}", line);
        }
    }

    println!("\n--- BFT METRICS ---");
    for line in body.lines() {
        if line.contains("bft") {
            println!("{}", line);
        }
    }

    println!("\n--- SYNC METRICS ---");
    for line in body.lines() {
        if line.contains("sync") {
            println!("{}", line);
        }
    }

    println!("\n--- NODE INFO METRICS ---");
    for line in body.lines() {
        if line.contains("node") || line.contains("uptime") {
            println!("{}", line);
        }
    }
}
