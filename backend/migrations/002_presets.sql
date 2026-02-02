-- WAVELET Database Migration
-- Version: 002 - Preset Enhancements
-- Created: 2026-02-02
-- Description: Adds enhanced preset management features including ratings and storage paths

-- ============================================
-- Presets Table Updates
-- ============================================

-- Add storage_path column if not exists
-- This stores the file path/URL to the preset file in storage
ALTER TABLE presets ADD COLUMN IF NOT EXISTS storage_path VARCHAR(500);

-- Add rating columns if not exist
-- Average rating (0.00 - 5.00)
ALTER TABLE presets ADD COLUMN IF NOT EXISTS rating DECIMAL(3,2) DEFAULT 0.00;

-- Number of ratings received
ALTER TABLE presets ADD COLUMN IF NOT EXISTS rating_count INTEGER DEFAULT 0;

-- ============================================
-- Indexes for Performance
-- ============================================

-- Index on category for filtering
CREATE INDEX IF NOT EXISTS idx_presets_category ON presets(category);

-- Partial index for public presets only (faster queries)
CREATE INDEX IF NOT EXISTS idx_presets_public ON presets(is_public) 
WHERE is_public = true;

-- Index on rating for sorting by rating
CREATE INDEX IF NOT EXISTS idx_presets_rating ON presets(rating DESC) 
WHERE rating > 0;

-- Index on download_count for sorting by popularity
CREATE INDEX IF NOT EXISTS idx_presets_downloads ON presets(download_count DESC);

-- ============================================
-- Preset Ratings Table
-- ============================================

-- Create preset_ratings table if not exists
-- Stores individual user ratings for presets
CREATE TABLE IF NOT EXISTS preset_ratings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    preset_id UUID NOT NULL REFERENCES presets(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(preset_id, user_id)
);

-- Indexes for ratings table
CREATE INDEX IF NOT EXISTS idx_ratings_preset_id ON preset_ratings(preset_id);
CREATE INDEX IF NOT EXISTS idx_ratings_user_id ON preset_ratings(user_id);
CREATE INDEX IF NOT EXISTS idx_ratings_created_at ON preset_ratings(created_at DESC);

-- ============================================
-- Update Function
-- ============================================

-- Create or replace function to update timestamp
CREATE OR REPLACE FUNCTION update_preset_rating_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Trigger for preset_ratings updated_at
DROP TRIGGER IF EXISTS update_preset_ratings_updated_at ON preset_ratings;
CREATE TRIGGER update_preset_ratings_updated_at 
    BEFORE UPDATE ON preset_ratings
    FOR EACH ROW 
    EXECUTE FUNCTION update_preset_rating_updated_at();

-- ============================================
-- Data Migration (if needed)
-- ============================================

-- If existing ratings exist in presets table, migrate them to preset_ratings
-- This is a one-time migration step
DO $$
DECLARE
    p RECORD;
    r RECORD;
BEGIN
    -- Only run if there are old-style ratings
    IF EXISTS (SELECT 1 FROM presets WHERE rating > 0) THEN
        -- Create a sample rating entry for each preset with a rating
        FOR p IN SELECT id, user_id, rating FROM presets WHERE rating > 0 LOOP
            -- Insert a representative rating (in real scenario, this would need user mapping)
            -- This is just for schema compatibility
            INSERT INTO preset_ratings (preset_id, user_id, rating, comment)
            VALUES (p.id, p.user_id, p.rating, 'Migrated rating')
            ON CONFLICT (preset_id, user_id) DO NOTHING;
        END LOOP;
    END IF;
END;
$$;

-- ============================================
-- Comments for Documentation
-- ============================================

COMMENT ON TABLE presets IS 'Stores synth presets created by users';
COMMENT ON TABLE preset_ratings IS 'Stores user ratings and comments for presets';
COMMENT ON COLUMN presets.storage_path IS 'File path or URL to the preset file in storage';
COMMENT ON COLUMN presets.rating IS 'Average rating (1-5) calculated from preset_ratings';
COMMENT ON COLUMN presets.rating_count IS 'Number of ratings received';
COMMENT ON INDEX idx_presets_public IS 'Optimizes queries for public presets only';
COMMENT ON INDEX idx_presets_rating IS 'Optimizes sorting by rating';
COMMENT ON INDEX idx_presets_downloads IS 'Optimizes sorting by download count';
