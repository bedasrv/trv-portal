module("luci.controller.trv-portal", package.seeall)

function index()
    entry({"admin", "services", "trv-portal"},
        alias("admin", "services", "trv-portal", "status"),
        _("TRV-Portal"), 90)
    
    entry({"admin", "services", "trv-portal", "status"},
        template("trv-portal/status"),
        _("Status"), 10)
    
    entry({"admin", "services", "trv-portal", "settings"},
        cbi("trv-portal/settings"),
        _("Settings"), 20)
    
    entry({"admin", "services", "trv-portal", "clone"},
        form("trv-portal/clone"),
        _("MAC Clone"), 30)
end
