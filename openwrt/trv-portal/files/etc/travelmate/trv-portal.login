#!/bin/sh
# TRV-Portal travelmate hook — called when travelmate detects a captive portal
# travelmate passes the portal domain as $1

PORTAL_DOMAIN="$1"
CONFIG="/etc/config/trv-portal"
STATE_DIR="/var/run/trv-portal"

mkdir -p "$STATE_DIR"

# Set UCI state
uci -q set trv-portal.@global[0].portal_mode='portal'
uci -q set trv-portal.@global[0].portal_domain="$PORTAL_DOMAIN"
uci commit trv-portal

# Enter portal mode via Rust binary
/usr/bin/trv-portal-mode enter "$PORTAL_DOMAIN"

# Start detection poll in background
/usr/bin/trv-portal-detect &
