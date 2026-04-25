-- LuCI config model for trv-portal settings

local m = Map("trv-portal", translate("TRV-Portal"),
    translate("Captive portal passthrough settings for travel routers."))

local s = m:section(NamedSection, "global", "trv-portal",
    translate("General Settings"))

s:option(Flag, "enabled",
    translate("Enable"),
    translate("Enable captive portal passthrough"))

s:option(Flag, "auto_clone",
    translate("Auto MAC Clone"),
    translate("Automatically detect and clone the authenticated device's MAC address"))

s:option(Value, "clone_timeout",
    translate("Clone Timeout"),
    translate("Seconds to wait for auto MAC detection"))

s:option(Value, "portal_detect_urls",
    translate("Portal Detection URLs"),
    translate("Space-separated URLs used to detect if internet is accessible"))

-- Portal mode settings
local p = m:section(NamedSection, "portal", "portal_mode_settings",
    translate("Portal Gateway Page"))

p:option(Value, "gateway_title",
    translate("Gateway Title"),
    translate("Title shown on the portal gateway page"))

p:option(Value, "gateway_message",
    translate("Gateway Message"),
    translate("Message shown on the portal gateway page"))

p:option(Value, "gateway_button",
    translate("Button Text"),
    translate("Text for the button that opens the hotel portal"))

return m
