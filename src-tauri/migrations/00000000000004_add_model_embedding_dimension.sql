-- Migration: Add embedding_dimension column to models table
-- This enables storing the vector dimensions for embedding models

-- Add embedding_dimension column to store the vector dimensions for each model
ALTER TABLE models ADD COLUMN embedding_dimension INTEGER;