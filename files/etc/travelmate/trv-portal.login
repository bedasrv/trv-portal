#!/bin/sh
# trv-portal.login — Travelmate captive portal hook.
# Called by travelmate when captive portal detected.
# Args: portal_domain (via $1 or travelmate's script_args)

PORTAL_DOMAIN="${1:-}"
WAN_IFACE="wwan"
LAN_IP="192.168.1.1"

log() { logger -t trv-portal "$*"; echo "[trv-portal] $*"; }

if [ -z "$PORTAL_DOMAIN" ]; then
    log "ERROR: no portal_domain provided"
    exit 255
fi

log "portal detected: $PORTAL_DOMAIN"

# Enter portal mode
if ! trv-portal-mode enter "$PORTAL_DOMAIN" "$WAN_IFACE" "$LAN_IP" 80; then
    log "ERROR: portal mode enter failed"
    exit 255
fi

# Poll loop — wait for internet
TIMEOUT=120
ELAPSED=0
while [ $ELAPSED -lt $TIMEOUT ]; do
    if trv-portal-detect; then
        log "internet detected after ${ELAPSED}s"
        break
    fi
    sleep 2
    ELAPSED=$((ELAPSED + 2))
done

if [ $ELAPSED -ge $TIMEOUT ]; then
    log "timeout waiting for auth — staying in portal mode"
    exit 255
fi

# Auto MAC clone
if trv-portal-clone auto "$WAN_IFACE"; then
    log "MAC cloned successfully"
else
    log "auto MAC clone failed — manual fallback needed"
fi

# Exit portal mode
if trv-portal-mode exit; then
    log "portal solved, internet OK"
    exit 0
else
    log "portal mode exit failed (non-fatal)"
    exit 0
fi
