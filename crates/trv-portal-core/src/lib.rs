// trv-portal-core: Zero-dependency system wrappers for OpenWrt captive portal.
//
// All functions call standard OpenWrt utilities (uci, iptables, dnsmasq, conntrack, ip).
// Designed for aarch64-unknown-linux-musl. No async, no external crates.

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Output};

// ── UCI ───────────────────────────────────────────────────────────

pub fn uci_get(key: &str) -> io::Result<String> {
    let out = cmd("uci", &["-q", "get", key])?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

pub fn uci_set(key: &str, value: &str) -> io::Result<()> {
    cmd_ok("uci", &["set", &format!("{key}={value}")])
}

pub fn uci_commit(config: &str) -> io::Result<()> {
    cmd_ok("uci", &["commit", config])
}

// ── DNSMASQ ───────────────────────────────────────────────────────

const DNSMASQ_CONF: &str = "/var/etc/dnsmasq.conf.trv-portal";

pub fn dnsmasq_write_hijack(lan_ip: &str, portal_domain: &str, hotel_dns: &str) -> io::Result<()> {
    let content = format!(
        "# trv-portal captive portal hijack\n\
         address=/#/{lan_ip}\n\
         server=/{portal_domain}/{hotel_dns}\n"
    );
    fs::write(DNSMASQ_CONF, content)
}

pub fn dnsmasq_clear_hijack() -> io::Result<()> {
    if Path::new(DNSMASQ_CONF).exists() {
        fs::remove_file(DNSMASQ_CONF)?;
    }
    Ok(())
}

pub fn dnsmasq_reload() -> io::Result<()> {
    cmd_ok("/etc/init.d/dnsmasq", &["reload"])
}

// ── IPTABLES ──────────────────────────────────────────────────────

pub fn iptables_add_redirect(lan_port: u16) -> io::Result<()> {
    cmd_ok(
        "iptables",
        &[
            "-t", "nat", "-I", "PREROUTING",
            "-p", "tcp", "--dport", "80",
            "-j", "REDIRECT", "--to-port", &lan_port.to_string(),
        ],
    )
}

pub fn iptables_del_redirect(lan_port: u16) -> io::Result<()> {
    cmd_ok(
        "iptables",
        &[
            "-t", "nat", "-D", "PREROUTING",
            "-p", "tcp", "--dport", "80",
            "-j", "REDIRECT", "--to-port", &lan_port.to_string(),
        ],
    )
}

// ── CONNTRACK ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LanDevice {
    pub ip: String,
    pub conn_count: usize,
    pub mac: Option<String>,
}

/// Find LAN devices that recently spoke TCP/443 through the WAN IP.
pub fn conntrack_find_lan(wan_ip: &str) -> io::Result<Vec<LanDevice>> {
    let out = cmd("conntrack", &["-L", "-p", "tcp", "--dport", "443"])?;
    let output = String::from_utf8_lossy(&out.stdout);

    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for line in output.lines() {
        // Extract src= field (original source = LAN device)
        for token in line.split_whitespace() {
            if token.starts_with("src=") {
                let ip = &token[4..];
                if !ip.starts_with(wan_ip) {
                    *counts.entry(ip.to_string()).or_default() += 1;
                }
            }
        }
    }

    let mut devices: Vec<LanDevice> = counts
        .into_iter()
        .map(|(ip, conn_count)| {
            let mac = ip_neigh(&ip).ok();
            LanDevice {
                ip,
                conn_count,
                mac,
            }
        })
        .collect();

    devices.sort_by(|a, b| b.conn_count.cmp(&a.conn_count));
    Ok(devices)
}

// ── IP NEIGH ──────────────────────────────────────────────────────

pub fn ip_neigh(ip: &str) -> io::Result<String> {
    let out = cmd("ip", &["neigh", "show", ip])?;
    let output = String::from_utf8_lossy(&out.stdout);
    // Format: "192.168.1.5 dev br-lan lladdr aa:bb:cc:dd:ee:ff REACHABLE"
    for token in output.split_whitespace() {
        if token == "lladdr" {
            // Next token is the MAC
            if let Some(mac) = output.split_whitespace().nth(
                output.split_whitespace().position(|t| t == "lladdr").unwrap() + 1,
            ) {
                return Ok(mac.to_string());
            }
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, format!("no MAC for {ip}")))
}

// ── INTERFACE CONTROL ─────────────────────────────────────────────

pub fn ifdown(iface: &str) -> io::Result<()> {
    cmd_ok("ifdown", &[iface])
}

pub fn ifup(iface: &str) -> io::Result<()> {
    cmd_ok("ifup", &[iface])
}

// ── DHCP DNS EXTRACTION ───────────────────────────────────────────

pub fn get_dhcp_dns(iface: &str) -> io::Result<String> {
    // Extract DNS from DHCP lease file
    let lease_path = format!("/tmp/resolv.conf.d/resolv.conf.auto");
    if let Ok(content) = fs::read_to_string(&lease_path) {
        for line in content.lines() {
            if line.starts_with("nameserver") {
                if let Some(dns) = line.split_whitespace().nth(1) {
                    return Ok(dns.to_string());
                }
            }
        }
    }
    // Fallback: try udhcpc lease
    let lease_file = format!("/var/run/udhcpc-{iface}.info");
    if let Ok(content) = fs::read_to_string(&lease_file) {
        for line in content.lines() {
            if line.starts_with("dns=") {
                return Ok(line[4..].to_string());
            }
        }
    }
    Ok("8.8.8.8".to_string()) // FALLBACK
}

// ── STATE FILE ────────────────────────────────────────────────────

const STATE_DIR: &str = "/var/run/trv-portal";

pub fn ensure_state_dir() -> io::Result<()> {
    fs::create_dir_all(STATE_DIR)
}

pub fn write_state(key: &str, value: &str) -> io::Result<()> {
    ensure_state_dir()?;
    fs::write(format!("{STATE_DIR}/{key}"), value)
}

pub fn read_state(key: &str) -> io::Result<String> {
    Ok(fs::read_to_string(format!("{STATE_DIR}/{key}"))?.trim().to_string())
}

// ── WAN IP ────────────────────────────────────────────────────────

pub fn get_wan_ip(iface: &str) -> io::Result<String> {
    let out = cmd("ip", &["-4", "addr", "show", "dev", iface])?;
    let output = String::from_utf8_lossy(&out.stdout);
    for line in output.lines() {
        if let Some(tok) = line.split_whitespace().find(|t| t.contains('/')) {
            let ip = tok.split('/').next().unwrap_or("");
            if ip.contains('.') {
                return Ok(ip.to_string());
            }
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, format!("no IP on {iface}")))
}

// ── HELPERS ───────────────────────────────────────────────────────

fn cmd<S: AsRef<OsStr>>(program: &str, args: &[S]) -> io::Result<Output> {
    Command::new(program)
        .args(args)
        .output()
}

fn cmd_ok<S: AsRef<OsStr>>(program: &str, args: &[S]) -> io::Result<()> {
    let out = cmd(program, args)?;
    if out.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr);
        Err(io::Error::new(io::ErrorKind::Other, stderr.trim().to_string()))
    }
}
