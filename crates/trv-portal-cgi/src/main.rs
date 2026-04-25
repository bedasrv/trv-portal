// trv-portal-cgi: uhttpd CGI handler for captive portal gateway.
//
// Endpoints (via PATH_INFO):
//   /cgi-bin/trv-portal/status  → JSON status
//   /cgi-bin/trv-portal/gateway → Serve gateway.html
//   /cgi-bin/trv-portal/portal?url=... → HTTP 307 redirect to hotel portal
//   /cgi-bin/trv-portal/clone?ip=X.X.X.X → Manual MAC clone
//
// Reads QUERY_STRING, PATH_INFO from CGI env.
// Responds with CGI headers + body on stdout.

use std::collections::HashMap;
use std::env;
use std::fs;

use trv_portal_core::*;

#[derive(serde::Serialize)]
struct Status {
    mode: String,
    portal_domain: String,
    portal_url: String,
    last_clone_mac: String,
    message: String,
}

fn main() {
    let path = env::var("PATH_INFO").unwrap_or_default();
    let query = parse_query_string(&env::var("QUERY_STRING").unwrap_or_default());

    match path.as_str() {
        "/status" | "/status.json" => handle_status(),
        "/gateway" => handle_gateway(),
        "/portal" => handle_portal_redirect(&query),
        "/clone" => handle_clone(&query),
        _ => handle_gateway(), // default: show gateway
    }
}

fn handle_status() {
    let mode = uci_get("trv-portal.@global[0].portal_mode")
        .unwrap_or_else(|_| "unknown".into());
    let portal_domain = uci_get("trv-portal.@global[0].portal_domain")
        .unwrap_or_default();
    let portal_url = uci_get("trv-portal.@global[0].portal_url")
        .unwrap_or_default();
    let last_clone_mac = uci_get("trv-portal.@global[0].last_clone_mac")
        .unwrap_or_default();

    let status = Status {
        mode: mode.clone(),
        portal_domain,
        portal_url,
        last_clone_mac,
        message: if mode == "portal" {
            "Waiting for authentication...".into()
        } else {
            "Normal operation".into()
        },
    };

    json_response(200, &status);
}

fn handle_gateway() {
    // Try to serve gateway.html from www
    let gateway_html = fs::read_to_string("/www/trv-portal/gateway.html")
        .unwrap_or_else(|_| {
            // Fallback inline gateway page
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Hotel WiFi Authentication</title>
<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
       background: #1a1a2e; color: #e0e0e0; display: flex; justify-content: center;
       align-items: center; min-height: 100vh; margin: 0; }
.card { background: #16213e; border-radius: 12px; padding: 2rem;
        max-width: 400px; width: 90%; text-align: center; box-shadow: 0 4px 24px rgba(0,0,0,0.4); }
h1 { font-size: 1.4rem; margin-bottom: 0.5rem; color: #e94560; }
p { margin: 0.75rem 0; font-size: 0.95rem; opacity: 0.9; }
.btn { display: inline-block; background: #e94560; color: #fff; border: none;
       padding: 0.8rem 2rem; border-radius: 8px; font-size: 1rem; cursor: pointer;
       text-decoration: none; margin: 0.5rem 0; font-weight: 600; }
.btn:hover { background: #c23152; }
.status { font-size: 0.85rem; opacity: 0.7; margin-top: 1rem; }
.admin-link { display: block; margin-top: 1.5rem; font-size: 0.8rem;
              color: #e94560; text-decoration: none; }
</style>
</head>
<body>
<div class="card">
  <h1>&#x1F510; Hotel WiFi</h1>
  <p>This network requires authentication before use.</p>
  <a class="btn" href="/cgi-bin/trv-portal/portal">Open Hotel Portal</a>
  <p class="status" id="status">Waiting for you to authenticate...</p>
  <a class="admin-link" href="/cgi-bin/luci">Router Admin &rarr;</a>
</div>
<script>
setInterval(async () => {
  try {
    const resp = await fetch('/cgi-bin/trv-portal/status');
    const data = await resp.json();
    if (data.mode === 'normal') {
      document.getElementById('status').textContent = '\u2713 Connected! All devices now have internet.';
      document.getElementById('status').style.color = '#4ade80';
    }
  } catch(e) {}
}, 2000);
</script>
</body>
</html>"#.into()
        });

    html_response(200, &gateway_html);
}

fn handle_portal_redirect(query: &HashMap<String, String>) {
    let portal_url = query.get("url").cloned().unwrap_or_else(|| {
        uci_get("trv-portal.@global[0].portal_url").unwrap_or_else(|_| {
            let domain = uci_get("trv-portal.@global[0].portal_domain").unwrap_or_default();
            if domain.is_empty() {
                "https://example.com".into()
            } else {
                format!("https://{domain}/login")
            }
        })
    });

    // HTTP 307 Temporary Redirect — preserves method
    print!("Status: 307 Temporary Redirect\r\n");
    print!("Location: {portal_url}\r\n");
    print!("Content-Type: text/plain\r\n\r\n");
    println!("Redirecting to {portal_url}");
}

fn handle_clone(query: &HashMap<String, String>) {
    if let Some(ip) = query.get("ip") {
        eprintln!("cgi: manual MAC clone requested for {ip}");
        match ip_neigh(ip) {
            Ok(mac) => {
                let wan_iface = read_state("wan_iface").unwrap_or_else(|_| "wwan".into());
                let _ = uci_set(&format!("network.{wan_iface}.macaddr"), &mac);
                let _ = uci_commit("network");
                let _ = write_state("cloned_mac", &mac);
                json_response(200, &serde_json::json!({
                    "status": "ok",
                    "message": format!("MAC cloned: {mac}"),
                    "mac": mac
                }));
            }
            Err(e) => {
                json_response(400, &serde_json::json!({
                    "status": "error",
                    "message": format!("Failed: {e}")
                }));
            }
        }
    } else {
        json_response(400, &serde_json::json!({
            "status": "error",
            "message": "Missing 'ip' parameter"
        }));
    }
}

// ── CGI helpers ───────────────────────────────────────────────────

fn html_response(code: u16, body: &str) {
    print!("Status: {code} OK\r\n");
    print!("Content-Type: text/html; charset=utf-8\r\n");
    print!("Cache-Control: no-cache\r\n\r\n");
    println!("{body}");
}

fn json_response<T: serde::Serialize>(code: u16, body: &T) {
    let json = serde_json::to_string(body).unwrap_or_default();
    print!("Status: {code} OK\r\n");
    print!("Content-Type: application/json\r\n");
    print!("Cache-Control: no-cache\r\n\r\n");
    println!("{json}");
}

fn parse_query_string(qs: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in qs.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            map.insert(
                url_decode(k),
                url_decode(v),
            );
        }
    }
    map
}

fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        match c {
            '+' => result.push(' '),
            '%' => {
                let hex = chars.by_ref().take(2).collect::<String>();
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                }
            }
            _ => result.push(c),
        }
    }
    result
}
