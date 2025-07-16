-- Add proxy_settings JSON column and drop individual proxy columns
ALTER TABLE model_providers 
ADD COLUMN proxy_settings JSONB DEFAULT '{}';

-- Migrate existing proxy data to JSON format
UPDATE model_providers 
SET proxy_settings = jsonb_build_object(
    'enabled', proxy_enabled,
    'url', proxy_url,
    'username', proxy_username,
    'password', proxy_password,
    'no_proxy', proxy_no_proxy,
    'ignore_ssl_certificates', proxy_ignore_ssl_certificates,
    'proxy_ssl', proxy_ssl,
    'proxy_host_ssl', proxy_host_ssl,
    'peer_ssl', proxy_peer_ssl,
    'host_ssl_verify', proxy_host_ssl_verify
);

-- Drop individual proxy columns
ALTER TABLE model_providers 
DROP COLUMN proxy_enabled,
DROP COLUMN proxy_url,
DROP COLUMN proxy_username,
DROP COLUMN proxy_password,
DROP COLUMN proxy_no_proxy,
DROP COLUMN proxy_ignore_ssl_certificates,
DROP COLUMN proxy_ssl,
DROP COLUMN proxy_host_ssl,
DROP COLUMN proxy_peer_ssl,
DROP COLUMN proxy_host_ssl_verify;