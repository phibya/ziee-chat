-- Add HTTP proxy settings to configurations table
-- This migration adds default proxy configuration for both global and user settings

-- Insert default global proxy configuration (for web app admin)
INSERT INTO configurations (name, value, description) VALUES 
    ('proxy.enabled', 'false', 'Enable global HTTP proxy for the application'),
    ('proxy.url', '', 'Global HTTP proxy URL'),
    ('proxy.username', '', 'Global HTTP proxy username'),
    ('proxy.password', '', 'Global HTTP proxy password'),
    ('proxy.noProxy', '', 'Global HTTP proxy no-proxy list (comma-separated)'),
    ('proxy.ignoreSslCertificates', 'false', 'Ignore SSL certificates for proxy'),
    ('proxy.proxySsl', 'false', 'Validate SSL certificate when connecting to proxy'),
    ('proxy.proxyHostSsl', 'false', 'Validate SSL certificate of proxy host'),
    ('proxy.peerSsl', 'false', 'Validate SSL certificates of peer connections'),
    ('proxy.hostSsl', 'false', 'Validate SSL certificates of destination hosts');

-- Add comment to document the proxy settings
COMMENT ON TABLE configurations IS 'Application configuration settings including appearance defaults, system settings, and HTTP proxy settings';