// trv-portal-clone: Detect authenticated device and clone its MAC to WAN.
//
// Usage:
//   trv-portal-clone auto <wan_iface>
//   trv-portal-clone manual <lan_ip> <wan_iface>

use std::process;
use trv_portal_core::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: trv-portal-clone <auto|manual> <wan_iface> [lan_ip]");
        process::exit(2);
    }

    let mode = &args[1];
    let wan_iface = &args[2];

    match mode.as_str() {
        "auto" => clone_auto(wan_iface),
        "manual" => {
            if args.len() < 4 {
                eprintln!("clone: manual requires <lan_ip>");
                process::exit(2);
            }
            clone_manual(&args[3], wan_iface);
        }
        other => {
            eprintln!("clone: unknown mode {other}");
            process::exit(2);
        }
    }
}

fn clone_auto(wan_iface: &str) {
    eprintln!("clone: auto-detecting authenticated device...");

    let wan_ip = match get_wan_ip(wan_iface) {
        Ok(ip) => ip,
        Err(e) => {
            eprintln!("clone: cannot get WAN IP: {e}");
            process::exit(1);
        }
    };

    let devices = match conntrack_find_lan(&wan_ip) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("clone: conntrack failed: {e}");
            process::exit(1);
        }
    };

    let best = match devices.into_iter().find(|d| d.mac.is_some()) {
        Some(d) => d,
        None => {
            eprintln!("clone: no device with known MAC found");
            process::exit(1);
        }
    };

    let mac = best.mac.as_ref().unwrap();
    eprintln!("clone: detected device {} ({} connections) → MAC {mac}", best.ip, best.conn_count);

    do_clone(mac, wan_iface);
}

fn clone_manual(lan_ip: &str, wan_iface: &str) {
    eprintln!("clone: manual — looking up MAC for {lan_ip}");

    let mac = match ip_neigh(lan_ip) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("clone: cannot resolve {lan_ip}: {e}");
            process::exit(1);
        }
    };

    eprintln!("clone: resolved {lan_ip} → MAC {mac}");
    do_clone(&mac, wan_iface);
}

fn do_clone(mac: &str, wan_iface: &str) {
    // Determine UCI network section from interface
    let network = if wan_iface.starts_with("wwan") { "wwan" } else { "wan" };

    eprintln!("clone: setting network.{network}.macaddr = {mac}");

    if let Err(e) = uci_set(&format!("network.{network}.macaddr"), mac) {
        eprintln!("clone: uci set failed: {e}");
        process::exit(1);
    }
    if let Err(e) = uci_commit("network") {
        eprintln!("clone: uci commit failed: {e}");
        process::exit(1);
    }

    // Save last cloned MAC
    let _ = uci_set("trv-portal.@global[0].last_clone_mac", mac);
    let _ = uci_commit("trv-portal");
    let _ = write_state("cloned_mac", mac);

    // Bounce interface — mwan3 handles failover during downtime
    eprintln!("clone: bouncing {wan_iface}...");
    if let Err(e) = ifdown(wan_iface) { eprintln!("clone: warn ifdown: {e}"); }
    std::thread::sleep(std::time::Duration::from_secs(1));
    if let Err(e) = ifup(wan_iface) { eprintln!("clone: warn ifup: {e}"); }

    eprintln!("clone: done — {wan_iface} now using MAC {mac}");
    println!("{mac}"); // stdout for scripting
}
