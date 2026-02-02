#!/bin/bash
# WAVELET Backend API Test Script
# Tests all API endpoints

BASE_URL="http://localhost:8080"

echo "🧪 WAVELET Backend API Test"
echo "============================"

# 1. Health Check
echo ""
echo "1. Health Check..."
curl -s $BASE_URL/health | jq .
sleep 1

# 2. Register a test user
echo ""
echo "2. Register test user..."
REGISTER_RESPONSE=$(curl -s -X POST $BASE_URL/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "email": "test@example.com", "password": "testpass123"}')
echo $REGISTER_RESPONSE | jq .

# Extract token from login (need to login first, but registration succeeded)

# 3. Login (uncomment to test with real credentials)
echo ""
echo "3. Login..."
LOGIN_RESPONSE=$(curl -s -X POST $BASE_URL/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "password": "testpass123"}')
echo $LOGIN_RESPONSE | jq .

# Extract token
TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.token // empty')
if [ -z "$TOKEN" ]; then
  echo "❌ Login failed, skipping authenticated tests"
  exit 1
fi

echo "✓ Logged in successfully"
echo "Token: ${TOKEN:0:20}..."

# 4. Get user profile
echo ""
echo "4. Get user profile..."
curl -s -H "Authorization: Bearer $TOKEN" $BASE_URL/api/users/1 | jq .

# 5. Create a preset
echo ""
echo "5. Create preset..."
PRESET_RESPONSE=$(curl -s -X POST $BASE_URL/api/presets \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "My Test Preset",
    "description": "A cool synth preset for testing",
    "category": "Lead",
    "tags": ["synth", "test"],
    "is_public": true,
    "preset_data": {
      "oscillator": {"waveform": "sawtooth", "frequency": 440},
      "filter": {"type": "lowpass", "cutoff": 2000}
    }
  }')
echo $PRESET_RESPONSE | jq .

PRESET_ID=$(echo $PRESET_RESPONSE | jq -r '.id // empty')

# 6. Search presets
echo ""
echo "6. Search presets..."
curl -s "$BASE_URL/api/presets?q=test" | jq .

# 7. Get feed (latest)
echo ""
echo "7. Get community feed (latest)..."
curl -s "$BASE_URL/api/feed?feed_type=latest&limit=5" | jq .

# 8. Get featured
echo ""
echo "8. Get featured presets..."
curl -s $BASE_URL/api/feed/featured | jq .

# 9. Get trending
echo ""
echo "9. Get trending presets..."
curl -s $BASE_URL/api/feed/trending | jq .

# 10. Rate preset
if [ -n "$PRESET_ID" ]; then
  echo ""
  echo "10. Rate preset..."
  curl -s -X POST $BASE_URL/api/presets/$PRESET_ID/rate \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $TOKEN" \
    -d '{"rating": 5, "comment": "Great preset!"}' | jq .
fi

echo ""
echo "============================"
echo "✅ All tests completed!"
