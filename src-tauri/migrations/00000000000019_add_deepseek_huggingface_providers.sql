-- Migration: Add DeepSeek and Hugging Face AI providers
-- This migration adds support for DeepSeek and Hugging Face as default AI providers

-- First, drop the existing CHECK constraint on provider_type
ALTER TABLE providers DROP CONSTRAINT providers_provider_type_check;

-- Add the new CHECK constraint with the additional provider types
ALTER TABLE providers ADD CONSTRAINT providers_provider_type_check 
    CHECK (provider_type IN ('local', 'openai', 'anthropic', 'groq', 'gemini', 'mistral', 'deepseek', 'huggingface', 'custom'));

-- Insert new provider records for DeepSeek and Hugging Face
-- Note: providers table doesn't have unique constraint on name, so we check for existence first
DO $$
BEGIN
    -- Insert DeepSeek provider if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM providers WHERE name = 'DeepSeek') THEN
        INSERT INTO providers (name, provider_type, enabled, built_in, base_url) 
        VALUES ('DeepSeek', 'deepseek', false, true, 'https://api.deepseek.com/v1');
    END IF;
    
    -- Insert Hugging Face provider if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM providers WHERE name = 'Hugging Face') THEN
        INSERT INTO providers (name, provider_type, enabled, built_in, base_url) 
        VALUES ('Hugging Face', 'huggingface', false, true, 'https://api-inference.huggingface.co/v1');
    END IF;
END $$;

-- Update any existing admin group to include access to all providers
-- This ensures admins can access the new providers
DO $$
DECLARE
    admin_group_id UUID;
    deepseek_provider_id UUID;
    huggingface_provider_id UUID;
BEGIN
    -- Get the admin group ID
    SELECT id INTO admin_group_id FROM user_groups WHERE name = 'admin';
    
    -- Get the new provider IDs
    SELECT id INTO deepseek_provider_id FROM providers WHERE name = 'DeepSeek';
    SELECT id INTO huggingface_provider_id FROM providers WHERE name = 'Hugging Face';
    
    -- Add providers to admin group if they exist
    IF admin_group_id IS NOT NULL AND deepseek_provider_id IS NOT NULL THEN
        INSERT INTO user_group_providers (group_id, provider_id)
        VALUES (admin_group_id, deepseek_provider_id)
        ON CONFLICT (group_id, provider_id) DO NOTHING;
    END IF;
    
    IF admin_group_id IS NOT NULL AND huggingface_provider_id IS NOT NULL THEN
        INSERT INTO user_group_providers (group_id, provider_id)
        VALUES (admin_group_id, huggingface_provider_id)
        ON CONFLICT (group_id, provider_id) DO NOTHING;
    END IF;
END $$;