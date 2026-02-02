# WAVELET Backend API Documentation

**Version**: 0.1.0
**Base URL**: `http://localhost:8080`
**Database**: PostgreSQL (wavelet)

---

## 🔐 Authentication

All protected endpoints require a JWT token in the Authorization header:
```
Authorization: Bearer <token>
```

Tokens are obtained via login and expire after 7 days.

---

## 📋 Endpoints

### Health Check

**GET** `/health`

Response:
```json
{
  "status": "healthy",
  "service": "wavelet-backend",
  "version": "0.1.0"
}
```

---

### 👤 Users

#### Get User Profile
**GET** `/api/users/{id}`

Headers: `Authorization: Bearer <token>`

Response:
```json
{
  "id": "uuid",
  "username": "testuser",
  "email": "test@example.com",
  "display_name": "Test User",
  "bio": null,
  "avatar_url": null,
  "presets_count": 5,
  "followers_count": 0,
  "following_count": 0,
  "created_at": "2026-02-02T13:21:48Z"
}
```

#### Get User Presets
**GET** `/api/users/{user_id}/presets`

Query Parameters:
- `page` (default: 1)
- `limit` (default: 20)
- `sort` (optional: "newest", "popular", "rating")

Response:
```json
{
  "presets": [...],
  "total": 5,
  "page": 1,
  "limit": 20,
  "total_pages": 1
}
```

---

### 🎹 Presets

#### Create Preset
**POST** `/api/presets`

Headers: `Authorization: Bearer <token>`

Body:
```json
{
  "name": "My Preset",
  "description": "A cool synth preset",
  "category": "Lead",
  "tags": ["synth", "electronic"],
  "is_public": true,
  "preset_data": {
    "oscillator": {
      "waveform": "sawtooth",
      "frequency": 440
    },
    "filter": {
      "type": "lowpass",
      "cutoff": 2000
    }
  }
}
```

Response (201 Created):
```json
{
  "id": "uuid",
  "name": "My Preset",
  "category": "Lead",
  "description": "A cool synth preset",
  "preset_data": {...},
  "author_id": "uuid",
  "author_name": "testuser",
  "author_username": "testuser",
  "downloads_count": 0,
  "rating": 0.0,
  "rating_count": 0,
  "is_public": true,
  "created_at": "2026-02-02T13:48:27Z",
  "updated_at": "2026-02-02T13:48:27Z"
}
```

#### Get Preset by ID
**GET** `/api/presets/{id}`

Response:
```json
{
  "id": "uuid",
  "name": "My Preset",
  ...
}
```

#### Search Presets
**GET** `/api/presets`

Query Parameters:
- `q` (optional) - Search text
- `category` (optional) - Filter by category
- `sort` (optional) - "newest", "popular", "rating", "downloads"
- `page` (default: 1)
- `limit` (default: 20)

Response:
```json
{
  "presets": [
    {
      "id": "uuid",
      "name": "Preset 1",
      "category": "Lead",
      "author_name": "testuser",
      "downloads_count": 10,
      "rating": 4.5,
      "rating_count": 5,
      ...
    }
  ],
  "total": 1,
  "page": 1,
  "limit": 20,
  "total_pages": 1
}
```

#### Download Preset
**GET** `/api/presets/{id}/download`

Response: JSON file (preset_data only)

#### Update Preset
**PUT** `/api/presets/{id}`

Headers: `Authorization: Bearer <token>`

Body (all fields optional):
```json
{
  "name": "Updated Name",
  "description": "Updated description",
  "category": "Bass",
  "is_public": false
}
```

#### Delete Preset
**DELETE** `/api/presets/{id}`

Headers: `Authorization: Bearer <token>`

Response:
```json
{
  "message": "Preset deleted successfully"
}
```

#### Rate Preset
**POST** `/api/presets/{id}/rate`

Headers: `Authorization: Bearer <token>`

Body:
```json
{
  "rating": 5,
  "comment": "Great preset!"
}
```

Response:
```json
{
  "message": "Rating submitted successfully"
}
```

Error (400): Cannot rate your own preset

---

### 📰 Community Feed

#### Get Community Feed
**GET** `/api/feed`

Query Parameters:
- `feed_type` (optional) - "latest", "popular", "featured", "following" (default: "latest")
- `page` (optional, default: 1)
- `limit` (optional, default: 20, max: 100)
- `category` (optional) - Filter by category

Response:
```json
{
  "feed_type": "latest",
  "items": [
    {
      "id": "uuid",
      "name": "Preset Name",
      "description": "A cool synth preset",
      "category": "Lead",
      "author_id": "uuid",
      "author_name": "Username",
      "author_username": "username",
      "downloads_count": 10,
      "likes_count": 5,
      "rating": 4.5,
      "rating_count": 3,
      "created_at": "2026-02-02T13:21:48Z",
      "is_featured": false
    }
  ],
  "page": 1,
  "limit": 20,
  "total": 5
}
```

#### Get Featured Presets
**GET** `/api/feed/featured`

Response:
```json
{
  "items": [...],
  "total": 10
}
```

#### Get Trending Presets
**GET** `/api/feed/trending`

Response:
```json
{
  "items": [...],
  "total": 10
}
```

---

## 🔑 Response Codes

| Code | Meaning |
|------|---------|
| 200 | Success |
| 201 | Created |
| 400 | Bad Request / Validation Error |
| 401 | Unauthorized (missing/invalid token) |
| 403 | Forbidden (access denied) |
| 404 | Not Found |
| 500 | Internal Server Error |

---

## 📝 Categories

- Lead
- Bass
- Pad
- Fx
- Keys
- Drums
- Sequencer
- Other

---

## 🏗️ Database Schema

### Users Table
- id (UUID, PRIMARY KEY)
- username (VARCHAR)
- email (VARCHAR, UNIQUE)
- password_hash (VARCHAR)
- display_name (VARCHAR)
- bio (TEXT)
- avatar_url (VARCHAR)
- created_at (TIMESTAMP)
- updated_at (TIMESTAMP)

### Presets Table
- id (UUID, PRIMARY KEY)
- user_id (UUID, FOREIGN KEY)
- name (VARCHAR)
- description (TEXT)
- category (VARCHAR)
- tags (JSONB)
- preset_data (JSONB)
- is_public (BOOLEAN)
- is_featured (BOOLEAN)
- downloads_count (INT)
- likes_count (INT)
- rating (DECIMAL)
- rating_count (INT)
- storage_path (VARCHAR)
- created_at (TIMESTAMP)
- updated_at (TIMESTAMP)

### Ratings Table
- id (UUID, PRIMARY KEY)
- preset_id (UUID, FOREIGN KEY)
- user_id (UUID, FOREIGN KEY)
- rating (INT)
- comment (TEXT)
- created_at (TIMESTAMP)

---

## 🏆 Challenges API

### Get Active Challenges
**GET** `/api/challenges`

Response:
```json
[
  {
    "id": 1,
    "title": "Retro Synth Wave Challenge",
    "description": "Create a synthwave track using only WAVELET presets! Show us your 80s vibes.",
    "theme": "80s Retro",
    "start_date": "2026-02-03T00:00:00Z",
    "end_date": "2026-02-10T23:59:59Z",
    "status": "active",
    "participant_count": 5,
    "created_by": 1,
    "created_at": "2026-02-03T00:00:00Z"
  }
]
```

### Create Challenge
**POST** `/api/challenges`

Headers: `Authorization: Bearer <token>`

Body:
```json
{
  "title": "Ambient Soundscapes",
  "description": "Create a relaxing ambient piece using WAVELET's reverb and delay effects.",
  "theme": "Peaceful Nature",
  "start_date": "2026-02-03T00:00:00Z",
  "end_date": "2026-02-17T23:59:59Z"
}
```

Response (201 Created):
```json
{
  "id": 2,
  "title": "Ambient Soundscapes",
  "description": "Create a relaxing ambient piece...",
  "theme": "Peaceful Nature",
  "start_date": "2026-02-03T00:00:00Z",
  "end_date": "2026-02-17T23:59:59Z",
  "status": "active",
  "participant_count": 0,
  "created_by": 1
}
```

### Get Challenge Details
**GET** `/api/challenges/{id}`

Response:
```json
{
  "id": 1,
  "title": "Retro Synth Wave Challenge",
  "description": "Create a synthwave track...",
  "theme": "80s Retro",
  "start_date": "2026-02-03T00:00:00Z",
  "end_date": "2026-02-10T23:59:59Z",
  "status": "active",
  "participant_count": 5,
  "created_by": 1,
  "created_at": "2026-02-03T00:00:00Z",
  "submissions": [
    {
      "id": 1,
      "challenge_id": 1,
      "user_id": 2,
      "username": "SynthMaster",
      "project_name": "Neon Nights",
      "description": "Pure 80s nostalgia with modern touch",
      "download_url": "/api/challenges/1/submissions/1/download",
      "votes": 42,
      "rank": 1,
      "submitted_at": "2026-02-03T12:00:00Z"
    }
  ]
}
```

### Get Challenge Leaderboard
**GET** `/api/challenges/{id}/leaderboard`

Response:
```json
[
  {
    "id": 1,
    "challenge_id": 1,
    "user_id": 2,
    "username": "SynthMaster",
    "project_name": "Neon Nights",
    "description": "Pure 80s nostalgia with modern touch",
    "download_url": "/api/challenges/1/submissions/1/download",
    "votes": 42,
    "rank": 1,
    "submitted_at": "2026-02-03T12:00:00Z"
  }
]
```

### Submit Project to Challenge
**POST** `/api/challenges/{challenge_id}/submissions`

Headers: `Authorization: Bearer <token>`

Body:
```json
{
  "project_name": "My Awesome Track",
  "description": "A synthwave track I created",
  "download_url": "http://localhost:8080/api/projects/download/abc123"
}
```

Response (201 Created):
```json
{
  "id": 1,
  "challenge_id": 1,
  "user_id": 3,
  "username": "testuser",
  "project_name": "My Awesome Track",
  "description": "A synthwave track I created",
  "download_url": "http://localhost:8080/api/projects/download/abc123",
  "votes": 0,
  "rank": 0,
  "submitted_at": "2026-02-03T12:00:00Z"
}
```

### Vote for Submission
**POST** `/api/challenges/{challenge_id}/submissions/{submission_id}/vote`

Headers: `Authorization: Bearer <token>`

Body:
```json
{
  "vote": true
}
```

Response:
```json
{
  "success": true,
  "message": "Vote recorded successfully"
}
```

---

### Challenges Database Schema

```sql
CREATE TABLE challenges (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    theme VARCHAR(255) NOT NULL,
    start_date TIMESTAMP WITH TIME ZONE NOT NULL,
    end_date TIMESTAMP WITH TIME ZONE NOT NULL,
    status VARCHAR(50) DEFAULT 'active',
    created_by INTEGER NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE challenge_submissions (
    id SERIAL PRIMARY KEY,
    challenge_id INTEGER NOT NULL REFERENCES challenges(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id),
    project_name VARCHAR(255) NOT NULL,
    description TEXT,
    download_url VARCHAR(500) NOT NULL,
    votes INTEGER DEFAULT 0,
    submitted_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(challenge_id, user_id)
);

CREATE TABLE challenge_votes (
    id SERIAL PRIMARY KEY,
    submission_id INTEGER NOT NULL REFERENCES challenge_submissions(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id),
    vote BOOLEAN NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(submission_id, user_id)
);
```

---

*Generated: 2026-02-03*
*WAVELET - Abstract Sound Synthesizer*
