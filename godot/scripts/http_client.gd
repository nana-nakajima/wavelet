extends Node
class_name WaveletHTTP

# HTTP client for WAVELET Backend API

const BASE_URL = "http://localhost:8080"
var http_request: HTTPRequest
var jwt_token: String = ""
var user_id: String = ""

signal request_completed(result: Dictionary)
signal auth_changed(is_logged_in: bool, user_data: Dictionary)

func _init():
	http_request = HTTPRequest.new()
	add_child(http_request)
	http_request.request_completed.connect(_on_request_completed)

func _ready():
	# Load saved token if exists
	_load_saved_token()

# ============ Authentication ============

func register(username: String, email: String, password: String) -> void:
	var body = {
		"username": username,
		"email": email,
		"password": password
	}
	_post("/api/auth/register", body)

func login(email: String, password: String) -> void:
	var body = {
		"email": email,
		"password": password
	}
	_post("/api/auth/login", body)

func logout():
	jwt_token = ""
	user_id = ""
	_save_token()
	auth_changed.emit(false, {})

func is_logged_in() -> bool:
	return jwt_token != ""

# ============ User ============

func get_user_profile(user_id: String) -> void:
	_get("/api/users/" + user_id)

func get_user_presets(user_id: String, page: int = 1, limit: int = 20) -> void:
	_get("/api/users/" + user_id + "/presets?page=" + str(page) + "&limit=" + str(limit))

# ============ Presets ============

func create_preset(name: String, description: String, category: String, 
                   preset_data: Dictionary, is_public: bool = true) -> void:
	var body = {
		"name": name,
		"description": description,
		"category": category,
		"is_public": is_public,
		"preset_data": preset_data
	}
	_post("/api/presets", body)

func get_preset(preset_id: String) -> void:
	_get("/api/presets/" + preset_id)

func search_presets(query: String = "", category: String = "", 
                    sort: String = "newest", page: int = 1, limit: int = 20) -> void:
	var url = "/api/presets?page=" + str(page) + "&limit=" + str(limit)
	if query:
		url += "&q=" + WWWFormData.new().add_text(query).data.uri_encode()
	if category:
		url += "&category=" + category
	if sort:
		url += "&sort=" + sort
	_get(url)

func download_preset(preset_id: String) -> void:
	_get("/api/presets/" + preset_id + "/download")

func update_preset(preset_id: String, data: Dictionary) -> void:
	_put("/api/presets/" + preset_id, data)

func delete_preset(preset_id: String) -> void:
	_delete("/api/presets/" + preset_id)

func rate_preset(preset_id: String, rating: int, comment: String = "") -> void:
	var body = {"rating": rating}
	if comment:
		body["comment"] = comment
	_post("/api/presets/" + preset_id + "/rate", body)

# ============ Share ============

func generate_share_link(preset_id: String) -> String:
	# Generate a shareable link for a preset
	return "https://wavelet.app/share/" + preset_id

func share_to_social(preset_id: String, platform: String) -> void:
	# Open share dialog for specific platform
	var share_url = generate_share_link(preset_id)
	match platform:
		"twitter":
			OS.shell_open("https://twitter.com/intent/tweet?text=Check+out+this+WAVELET+preset&url=" + share_url)
		"facebook":
			OS.shell_open("https://www.facebook.com/sharer/sharer.php?u=" + share_url)
		"reddit":
			OS.shell_open("https://reddit.com/submit?url=" + share_url)
		"copy":
			OS.set_clipboard(share_url)

# ============ Follow System ============

func follow_user(user_id: String) -> void:
	_post("/api/users/" + user_id + "/follow", {})

func unfollow_user(user_id: String) -> void:
	_delete("/api/users/" + user_id + "/follow")

func check_follow_status(user_id: String) -> void:
	_get("/api/users/" + user_id + "/follow/check")

func get_followers(user_id: String, page: int = 1, limit: int = 20) -> void:
	_get("/api/users/" + user_id + "/followers?page=" + str(page) + "&limit=" + str(limit))

func get_following(user_id: String, page: int = 1, limit: int = 20) -> void:
	_get("/api/users/" + user_id + "/following?page=" + str(page) + "&limit=" + str(limit))

# ============ Community Feed ============

func get_feed(feed_type: String = "latest", page: int = 1, limit: int = 20, 
              category: String = "") -> void:
	var url = "/api/feed?feed_type=" + feed_type + "&page=" + str(page) + "&limit=" + str(limit)
	if category:
		url += "&category=" + category
	_get(url)

func get_featured_presets() -> void:
	_get("/api/feed/featured")

func get_trending_presets() -> void:
	_get("/api/feed/trending")

# ============ HTTP Methods ============

func _get(endpoint: String) -> void:
	var headers = PackedStringArray()
	headers.append("Content-Type: application/json")
	if jwt_token:
		headers.append("Authorization: Bearer " + jwt_token)
	
	var error = http_request.request(BASE_URL + endpoint, headers, true, HTTPClient.METHOD_GET)
	if error != OK:
		request_completed.emit({"error": "Request failed", "code": error})

func _post(endpoint: String, body: Dictionary) -> void:
	var headers = PackedStringArray()
	headers.append("Content-Type: application/json")
	if jwt_token:
		headers.append("Authorization: Bearer " + jwt_token)
	
	var json = JSON.stringify(body)
	var error = http_request.request(BASE_URL + endpoint, headers, true, 
                                     HTTPClient.METHOD_POST, json)
	if error != OK:
		request_completed.emit({"error": "Request failed", "code": error})

func _put(endpoint: String, body: Dictionary) -> void:
	var headers = PackedStringArray()
	headers.append("Content-Type: application/json")
	if jwt_token:
		headers.append("Authorization: Bearer " + jwt_token)
	
	var json = JSON.stringify(body)
	var error = http_request.request(BASE_URL + endpoint, headers, true, 
                                     HTTPClient.METHOD_PUT, json)
	if error != OK:
		request_completed.emit({"error": "Request failed", "code": error})

func _delete(endpoint: String) -> void:
	var headers = PackedStringArray()
	headers.append("Content-Type: application/json")
	if jwt_token:
		headers.append("Authorization: Bearer " + jwt_token)
	
	var error = http_request.request(BASE_URL + endpoint, headers, true, HTTPClient.METHOD_DELETE)
	if error != OK:
		request_completed.emit({"error": "Request failed", "code": error})

func _on_request_completed(result: int, response_code: int, headers: PackedStringArray, body: PackedByteArray) -> void:
	var json = JSON.new()
	var error = json.parse_string(body.get_string_from_utf8())
	
	var response = {
		"result": result,
		"code": response_code,
		"data": json.get_data() if error == OK else {},
		"error": ""
	}
	
	if response_code != 200:
		if typeof(response["data"]) == TYPE_DICTIONARY and response["data"].has("error"):
			response["error"] = response["data"]["error"]
		else:
			response["error"] = "HTTP " + str(response_code)
	
	# Handle auth responses
	if result == OK:
		match response_code:
			201:  # Created (registration)
				if typeof(response["data"]) == TYPE_DICTIONARY:
					response["message"] = "Registration successful! Please login."
			200:  # Success
				if typeof(response["data"]) == TYPE_DICTIONARY:
					# Check for token in login response
					if response["data"].has("token"):
						jwt_token = response["data"]["token"]
						if response["data"].has("user"):
							user_id = response["data"]["user"].get("id", "")
						_save_token()
						auth_changed.emit(true, response["data"].get("user", {}))
					
					# Check for user data
					if response["data"].has("id") and response["data"].has("username"):
						response["user_data"] = response["data"]
	
	request_completed.emit(response)

# ============ Token Management ============

func _save_token():
	if jwt_token:
		var save_file = FileAccess.open("user_token.dat", FileAccess.WRITE)
		save_file.store_line(jwt_token)
		save_file.store_line(user_id)

func _load_saved_token():
	if FileAccess.file_exists("user_token.dat"):
		var save_file = FileAccess.open("user_token.dat", FileAccess.READ)
		jwt_token = save_file.get_line()
		user_id = save_file.get_line()
		if jwt_token:
			auth_changed.emit(true, {"id": user_id})

# ============ Utility ============

func get_error_message(response: Dictionary) -> String:
	if response.has("error") and response["error"]:
		return response["error"]
	match response.get("code", 0):
		401:
			return "Please login to continue"
		403:
			return "Access denied"
		404:
			return "Not found"
		500:
			return "Server error"
		_:
			return "An error occurred"
