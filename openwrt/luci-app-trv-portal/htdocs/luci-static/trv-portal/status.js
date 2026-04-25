// TRV-Portal status dashboard JS

'use strict';
'require uclient';
'require ui';

var statusEl, logEl;

function refreshStatus() {
    return uclient.get('/cgi-bin/trv-portal/status', null, function(res) {
        var data = JSON.parse(res);
        var modeClass = data.mode === 'portal' ? 'label-warning' : 'label-success';
        var modeIcon = data.mode === 'portal' ? '&#x26A0;' : '&#x2713;';

        statusEl.innerHTML = [
            '<table class="table">',
            '<tr><td><strong>Mode</strong></td><td><span class="label ' + modeClass + '">' + modeIcon + ' ' + data.mode + '</span></td></tr>',
            '<tr><td><strong>Portal Domain</strong></td><td>' + LUCI.escapeHTML(data.portal_domain || 'N/A') + '</td></tr>',
            '<tr><td><strong>Last Cloned MAC</strong></td><td><code>' + LUCI.escapeHTML(data.last_clone_mac || 'N/A') + '</code></td></tr>',
            '<tr><td><strong>Status</strong></td><td>' + LUCI.escapeHTML(data.message) + '</td></tr>',
            '</table>'
        ].join('');
    });
}

function renderStatus() {
    statusEl = E('div', { 'class': 'cbi-section' });
    refreshStatus();
    window.setInterval(refreshStatus, 3000);
    return statusEl;
}

return renderStatus;
