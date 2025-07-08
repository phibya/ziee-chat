-- Create user settings table to store personal user preferences
-- This table will store settings like appearance, shortcuts, proxy settings, etc.

CREATE TABLE user_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    value JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    UNIQUE(user_id, key)
);

-- Create indexes for better performance
CREATE INDEX idx_user_settings_user_id ON user_settings(user_id);
CREATE INDEX idx_user_settings_key ON user_settings(key);
CREATE INDEX idx_user_settings_user_id_key ON user_settings(user_id, key);
CREATE INDEX idx_user_settings_value ON user_settings USING GIN(value);

-- Create trigger for updated_at column
CREATE TRIGGER update_user_settings_updated_at
    BEFORE UPDATE ON user_settings
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Add comment to track the migration
COMMENT ON TABLE user_settings IS 'User settings table for storing personal preferences like appearance, shortcuts, proxy settings, etc.';
COMMENT ON COLUMN user_settings.key IS 'Setting key using camelCase format (e.g., "appearance.theme", "appearance.fontSize")';
COMMENT ON COLUMN user_settings.value IS 'Setting value stored as JSONB for flexibility';