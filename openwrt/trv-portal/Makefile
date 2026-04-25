include $(TOPDIR)/rules.mk

PKG_NAME:=trv-portal
PKG_VERSION:=0.1.0
PKG_RELEASE:=1

PKG_LICENSE:=MIT
PKG_MAINTAINER:=bedasrv

include $(INCLUDE_DIR)/package.mk

define Package/trv-portal
    SECTION:=net
    CATEGORY:=Network
    TITLE:=Captive Portal Passthrough for Travel Routers
    DEPENDS:=+iptables +conntrack +dnsmasq-full +travelmate +mwan3
    URL:=https://github.com/bedasrv/trv-portal
endef

define Package/trv-portal/description
Universal captive portal passthrough for OpenWrt travel routers.
DNS hijack + HTTP proxy + Lua handler to pass hotel portals to
user devices, with automatic MAC cloning via conntrack.
endef

define Build/Prepare
endef

define Build/Compile
	# Binaries are pre-built via GitHub Actions (Rust cross-compile)
	# They land in files/usr/bin/ via CI artifact staging
	cp -r $(PKG_BUILD_DIR)/files/* $(PKG_INSTALL_DIR)/ 2>/dev/null || true
endef

define Package/trv-portal/conffiles
/etc/config/trv-portal
endef

define Package/trv-portal/install
	$(INSTALL_DIR) $(1)/etc/config
	$(INSTALL_CONF) ./files/etc/config/trv-portal $(1)/etc/config/

	$(INSTALL_DIR) $(1)/etc/travelmate
	$(INSTALL_BIN) ./files/etc/travelmate/trv-portal.login $(1)/etc/travelmate/

	$(INSTALL_DIR) $(1)/etc/hotplug.d/iface
	$(INSTALL_BIN) ./files/etc/hotplug.d/iface/99-trv-portal $(1)/etc/hotplug.d/iface/

	$(INSTALL_DIR) $(1)/etc/uci-defaults
	$(INSTALL_BIN) ./files/etc/uci-defaults/99-trv-portal $(1)/etc/uci-defaults/

	$(INSTALL_DIR) $(1)/usr/bin
	# Binaries placed here by CI or manual build
	if [ -f ./files/usr/bin/trv-portal-detect ]; then \
		$(INSTALL_BIN) ./files/usr/bin/trv-portal-detect $(1)/usr/bin/; \
		$(INSTALL_BIN) ./files/usr/bin/trv-portal-mode $(1)/usr/bin/; \
		$(INSTALL_BIN) ./files/usr/bin/trv-portal-clone $(1)/usr/bin/; \
		$(INSTALL_BIN) ./files/usr/bin/trv-portal-cgi $(1)/usr/bin/; \
	fi

	$(INSTALL_DIR) $(1)/www/trv-portal
	$(INSTALL_DATA) ./files/www/trv-portal/* $(1)/www/trv-portal/

	$(INSTALL_DIR) $(1)/var/run/trv-portal
endef

$(eval $(call BuildPackage,trv-portal))
