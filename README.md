# TRV-Portal

**Universal captive portal passthrough for OpenWrt travel routers.**

Hotels block internet behind captive portals. TRV-Portal passes the portal to your device, you authenticate once, then it clones your MAC so all your devices get internet.

## How It Works

```
Hotel WiFi → travelmate detects portal → DNS hijack + HTTP redirect → 
You authenticate on phone → MAC cloned to router → everyone online
```

## Architecture

| Component | Language | Purpose |
|-----------|----------|---------|
| `trv-portal-core` | Rust (lib) | System wrappers: UCI, iptables, dnsmasq, conntrack |
| `trv-portal-detect` | Rust (bin) | Multi-URL internet reachability check |
| `trv-portal-mode` | Rust (bin) | Enter/exit portal mode (DNS + iptables) |
| `trv-portal-clone` | Rust (bin) | Auto-detect auth device → MAC clone |
| `trv-portal-cgi` | Rust (bin) | uhttpd CGI handler (status, redirect, gateway) |
| `trv-portal.login` | Shell | Travelmate hook (glue) |
| `luci-app-trv-portal` | Lua | LuCI web interface |

## Requirements

- OpenWrt SNAPSHOT (aarch64_cortex-a53, mediatek/filogic, musl)
- Kernel 6.12.74
- travelmate ≥ 2.4.0
- mwan3 (multi-WAN failover)
- dnsmasq-full
- iptables, conntrack

## Building

### 1. Rust Binaries

```bash
# Install cross
cargo install cross

# Build for OpenWrt (aarch64 musl)
cross build --release --target aarch64-unknown-linux-musl
```

### 2. OpenWrt Packages

```bash
# With OpenWrt SDK
cp -r openwrt/trv-portal $SDK_DIR/package/
cp openwrt/luci-app-trv-portal $SDK_DIR/package/
cd $SDK_DIR
make package/trv-portal/compile V=s -j$(nproc)
```

### 3. GitHub Actions (automatic)

Push to `main` → CI builds Rust binaries + OpenWrt IPKs → artifacts ready.

Tag `v*` → GitHub Release with all binaries and IPKs.

## Installation

```bash
# From APK
apk add trv-portal*.apk luci-app-trv-portal*.apk

# Or direct binary
scp trv-portal-* root@192.168.1.1:/usr/bin/
```

## Configuration

Web UI: LuCI → Services → TRV-Portal

Or via UCI:

```bash
uci set trv-portal.@global[0].enabled='1'
uci set trv-portal.@global[0].auto_clone='1'
uci commit trv-portal
```

## License

MIT
