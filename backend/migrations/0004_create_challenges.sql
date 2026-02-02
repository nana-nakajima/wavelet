-- Create challenges table
CREATE TABLE IF NOT EXISTS challenges (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    theme VARCHAR(255) NOT NULL,
    start_date TIMESTAMP WITH TIME ZONE NOT NULL,
    end_date TIMESTAMP WITH TIME ZONE NOT NULL,
    status VARCHAR(50) DEFAULT 'active',
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create challenge submissions table
CREATE TABLE IF NOT EXISTS challenge_submissions (
    id SERIAL PRIMARY KEY,
    challenge_id INTEGER NOT NULL REFERENCES challenges(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    project_name VARCHAR(255) NOT NULL,
    description TEXT,
    download_url VARCHAR(500) NOT NULL,
    votes INTEGER DEFAULT 0,
    submitted_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(challenge_id, user_id)
);

-- Create challenge votes table (prevent duplicate votes)
CREATE TABLE IF NOT EXISTS challenge_votes (
    id SERIAL PRIMARY KEY,
    submission_id INTEGER NOT NULL REFERENCES challenge_submissions(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    vote BOOLEAN NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(submission_id, user_id)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_challenges_status ON challenges(status);
CREATE INDEX IF NOT EXISTS idx_challenge_submissions_challenge_id ON challenge_submissions(challenge_id);
CREATE INDEX IF NOT EXISTS idx_challenge_submissions_votes ON challenge_submissions(challenge_id, votes DESC);
CREATE INDEX IF NOT EXISTS idx_challenge_votes_submission_id ON challenge_votes(submission_id);
