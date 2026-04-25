// trv-portal-detect: Check if internet is accessible (portal solved).
//
// Usage: trv-portal-detect [URL...]
//   Reads URLs from args, or falls back to UCI portal_detect_urls config.
//   Exits 0 if any URL returns HTTP 200 → internet open.
//   Exits 1 if all URLs fail → still captive.

use std::process;
use std::time::Duration;

fn main() {
    let urls = get_urls();
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(3))
        .timeout_read(Duration::from_secs(5))
        .build();

    for url in &urls {
        match agent.get(url).call() {
            Ok(resp) if resp.status() == 200 => {
                // Check for success content patterns
                if let Ok(body) = resp.into_string() {
                    let body = body.trim().to_lowercase();
                    if body.contains("success") || body.is_empty() || body.contains("204") {
                        eprintln!("detect: OK {url}");
                        process::exit(0);
                    }
                }
            }
            Ok(resp) => {
                eprintln!("detect: {url} → HTTP {}", resp.status());
            }
            Err(e) => {
                eprintln!("detect: {url} → {e}");
            }
        }
    }

    eprintln!("detect: all probes failed — still captive");
    process::exit(1);
}

fn get_urls() -> Vec<String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        return args;
    }

    // Fallback: UCI config
    let urls_str = trv_portal_core::uci_get("trv-portal.@global[0].portal_detect_urls")
        .unwrap_or_default();

    if !urls_str.is_empty() {
        return urls_str.split_whitespace().map(String::from).collect();
    }

    // Hardcoded fallback
    vec![
        "http://detectportal.firefox.com/success.txt".into(),
        "http://captive.apple.com/hotspot-detect.html".into(),
        "http://www.gstatic.com/generate_204".into(),
    ]
}
