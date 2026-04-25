include $(TOPDIR)/rules.mk

LUCI_TITLE:=LuCI support for TRV-Portal captive portal passthrough
LUCI_DEPENDS:=+luci-base +trv-portal
LUCI_PKGARCH:=all

PKG_NAME:=luci-app-trv-portal
PKG_VERSION:=0.1.0
PKG_RELEASE:=1
PKG_LICENSE:=MIT
PKG_MAINTAINER:=bedasrv

include $(TOPDIR)/feeds/luci/luci.mk

# Call BuildPackage - OpenWrt
$(eval $(call BuildPackage,luci-app-trv-portal))
