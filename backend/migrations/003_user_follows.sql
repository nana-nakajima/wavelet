-- Migration: Create user_follows table for community features
-- Created: 2026-02-02
-- Purpose: Support "following" feed and social features

-- User follows table (for "following" feed)
CREATE TABLE IF NOT EXISTS user_follows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    follower_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    following_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Ensure unique follow relationship
    CONSTRAINT unique_follow UNIQUE (follower_id, following_id)
);

-- Index for efficient feed queries
CREATE INDEX IF NOT EXISTS idx_user_follows_follower ON user_follows(follower_id);
CREATE INDEX IF NOT EXISTS idx_user_follows_following ON user_follows(following_id);
CREATE INDEX IF NOT EXISTS idx_user_follows_created ON user_follows(created_at DESC);

-- Add comment
COMMENT ON TABLE user_follows IS 'Tracks which users follow other users for the community feed';
COMMENT ON COLUMN user_follows.follower_id IS 'The user who is following';
COMMENT ON COLUMN user_follows.following_id IS 'The user being followed';
