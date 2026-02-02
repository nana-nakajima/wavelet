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

*Generated: 2026-02-02*
*WAVELET - Abstract Sound Synthesizer*
