-- Create configuration table
CREATE TABLE configurations (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    value TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create index on name for faster lookups
CREATE INDEX idx_configurations_name ON configurations(name);

-- Insert default configuration for initialization status
INSERT INTO configurations (name, value, description) VALUES 
    ('is_initialized', 'false', 'Indicates whether the application has been initialized'),
    ('enable_user_registration', 'true', 'Controls whether new user registration is enabled');