-- Migration: Create API Proxy Server tables and configuration
-- This migration creates the necessary tables and configuration for the API Proxy Server

-- Create api_proxy_server_models table
CREATE TABLE api_proxy_server_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_id UUID NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    alias_id VARCHAR(255) NULL,           -- Human-readable alias for the model
    enabled BOOLEAN NOT NULL DEFAULT true,
    is_default BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(model_id)
);

-- Ensure alias uniqueness (excluding NULL values)
CREATE UNIQUE INDEX idx_api_proxy_server_models_alias 
ON api_proxy_server_models (alias_id) 
WHERE alias_id IS NOT NULL;

-- Ensure only one default model at a time
CREATE UNIQUE INDEX idx_api_proxy_server_models_default 
ON api_proxy_server_models (is_default) 
WHERE is_default = true;

-- Create api_proxy_server_trusted_hosts table
CREATE TABLE api_proxy_server_trusted_hosts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    host VARCHAR(255) NOT NULL,           -- IP address, domain, or CIDR notation
    description TEXT NULL,                -- Optional description of the host
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(host)
);

-- Insert default trusted hosts
INSERT INTO api_proxy_server_trusted_hosts (host, description, enabled) VALUES
('127.0.0.1', 'Localhost IPv4', true),
('::1', 'Localhost IPv6', true),
('localhost', 'Localhost domain', true)
ON CONFLICT (host) DO NOTHING;

-- API Proxy Server configuration keys
INSERT INTO configurations (key, value, description) VALUES
('api_proxy_server_port', '"8080"', 'API Proxy Server Port'),
('api_proxy_server_address', '"127.0.0.1"', 'API Proxy Server Bind Address'),
('api_proxy_server_prefix', '"/v1"', 'API Proxy Server URL Prefix'),
('api_proxy_server_enabled', 'false', 'Enable/Disable API Proxy Server'),
('api_proxy_server_api_key', '""', 'API Key for Proxy Server Authentication'),
('api_proxy_server_allow_cors', 'true', 'Enable CORS for API Proxy Server'),
('api_proxy_server_log_level', '"info"', 'Log Level: error, warn, info, debug, trace')
ON CONFLICT (key) DO NOTHING;