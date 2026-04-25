// trv-portal-mode: Enter/exit captive portal passthrough mode.
//
// Usage:
//   trv-portal-mode enter <portal_domain> <wan_iface> <lan_ip> [lan_port]
//   trv-portal-mode exit

use std::process;

use trv_portal_core::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: trv-portal-mode <enter|exit> [args...]");
        process::exit(2);
    }

    match args[1].as_str() {
        "enter" => enter(&args),
        "exit" => exit_portal(),
        other => {
            eprintln!("unknown subcommand: {other}");
            process::exit(2);
        }
    }
}

fn enter(args: &[String]) {
    let portal_domain = args.get(2).map(|s| s.as_str()).unwrap_or("");
    let wan_iface = args.get(3).map(|s| s.as_str()).unwrap_or("wwan");
    let lan_ip = args.get(4).map(|s| s.as_str()).unwrap_or("192.168.1.1");
    let lan_port = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(80u16);

    if portal_domain.is_empty() {
        eprintln!("mode: portal_domain required for enter");
        process::exit(1);
    }

    eprintln!("mode: entering portal mode for {portal_domain}");

    // Save current state
    if let Ok(wan_ip) = get_wan_ip(wan_iface) {
        let _ = write_state("wan_iface", wan_iface);
        let _ = write_state("wan_ip", &wan_ip);
    }

    // Get hotel DNS
    let hotel_dns = get_dhcp_dns(wan_iface).unwrap_or_else(|_| "8.8.8.8".into());
    let _ = write_state("hotel_dns", &hotel_dns);

    // DNS hijack
    if let Err(e) = dnsmasq_write_hijack(lan_ip, portal_domain, &hotel_dns) {
        eprintln!("mode: dnsmasq config failed: {e}");
        process::exit(1);
    }
    if let Err(e) = dnsmasq_reload() {
        eprintln!("mode: dnsmasq reload failed: {e}");
        process::exit(1);
    }

    // iptables HTTP redirect
    if let Err(e) = iptables_add_redirect(lan_port) {
        eprintln!("mode: iptables redirect failed: {e}");
        process::exit(1);
    }

    // Set UCI flag
    let _ = uci_set("trv-portal.@global[0].portal_mode", "portal");
    let _ = uci_set("trv-portal.@global[0].portal_domain", portal_domain);
    let _ = uci_commit("trv-portal");

    // Write runtime state
    let _ = write_state("portal_mode", "active");
    let _ = write_state("portal_domain", portal_domain);

    eprintln!("mode: portal mode active — DNS hijack + iptables redirect enabled");
}

fn exit_portal() {
    let lan_port: u16 = 80;

    eprintln!("mode: exiting portal mode");

    // Remove DNS hijack
    if let Err(e) = dnsmasq_clear_hijack() {
        eprintln!("mode: warn: dnsmasq cleanup: {e}");
    }
    let _ = dnsmasq_reload();

    // Remove iptables
    if let Err(e) = iptables_del_redirect(lan_port) {
        eprintln!("mode: warn: iptables cleanup: {e}");
    }

    // Restore UCI
    let _ = uci_set("trv-portal.@global[0].portal_mode", "normal");
    let _ = uci_set("trv-portal.@global[0].portal_domain", "");
    let _ = uci_commit("trv-portal");

    // Clear runtime state
    let _ = write_state("portal_mode", "inactive");

    eprintln!("mode: portal mode exited — DNS and iptables restored");
}
