extends Control

# Community Panel - Browse and download community presets

@onready var http: WaveletHTTP = $"../HTTPClient"
@onready var preset_list: VBoxContainer = $Panel/ScrollContainer/PresetList
@onready var feed_type_option: OptionButton = $Panel/FeedControls/FeedType
@onready var search_input: LineEdit = $Panel/FeedControls/SearchInput
@onready var loading_label: Label = $Panel/LoadingLabel
@onready var login_panel: Panel = $LoginPanel
@onready var username_input: LineEdit = $LoginPanel/Form/Username
@onready var email_input: LineEdit = $LoginPanel/Form/Email
@onready var password_input: LineEdit = $LoginPanel/Form/Password
@onready var login_button: Button = $LoginPanel/Form/LoginButton
@onready var register_button: Button = $LoginPanel/Form/RegisterButton
@onready var logout_button: Button = $LoginPanel/LogoutButton
@onready var user_info_label: Label = $LoginPanel/UserInfo

var current_page: int = 1
var current_feed_type: String = "latest"
var current_category: String = ""
var presets_data: Array = []
var follow_states: Dictionary = {}  # user_id: is_following

func _ready():
	# Connect signals
	http.request_completed.connect(_on_request_completed)
	http.auth_changed.connect(_on_auth_changed)
	
	# Feed type options
	feed_type_option.add_item("Latest", 0)
	feed_type_option.add_item("Popular", 1)
	feed_type_option.add_item("Featured", 2)
	feed_type_option.add_item("Trending", 3)
	feed_type_option.item_selected.connect(_on_feed_type_selected)
	
	# Update UI based on auth state
	_on_auth_changed(http.is_logged_in(), {})
	
	# Load initial feed
	load_feed()

func _on_auth_changed(is_logged_in: bool, user_data: Dictionary):
	login_panel.visible = not is_logged_in
	logout_button.visible = is_logged_in
	
	if is_logged_in:
		user_info_label.text = "Logged in as: " + user_data.get("username", user_data.get("email", "User"))
		user_info_label.visible = true
	else:
		user_info_label.visible = false

func load_feed():
	loading_label.visible = true
	preset_list.visible = false
	http.get_feed(current_feed_type, current_page, 20, current_category)

func search_presets():
	var query = search_input.text.strip_edges()
	if query:
		http.search_presets(query, "", "newest", 1, 20)
	else:
		load_feed()

func _on_feed_type_selected(index: int):
	var types = ["latest", "popular", "featured", "trending"]
	current_feed_type = types[index] if index < types.size() else "latest"
	load_feed()

func _on_search_pressed():
	search_presets()

func _on_prev_page_pressed():
	if current_page > 1:
		current_page -= 1
		load_feed()

func _on_next_page_pressed():
	current_page += 1
	load_feed()

func _on_request_completed(response: Dictionary):
	loading_label.visible = false
	preset_list.visible = true
	
	if response.get("code") != 200:
		# Check if this is a follow status check
		if response.get("endpoint", "").ends_with("/follow/check"):
			# Ignore follow check errors (user might not exist)
			return
		
		_show_error(http.get_error_message(response))
		return
	
	var data = response.get("data", {})
	if typeof(data) != TYPE_DICTIONARY:
		# Check if this is a follow status check response (might be boolean)
		if typeof(data) == TYPE_BOOL:
			return
		_show_error("Invalid response format")
		return
	
	# Check for follow status response
	var endpoint = response.get("endpoint", "")
	if endpoint.ends_with("/follow/check"):
		var is_following = data.get("following", false)
		var user_id = ""
		# Extract user_id from endpoint
		var parts = endpoint.split("/")
		if parts.size() >= 3:
			user_id = parts[2]
		if user_id:
			follow_states[user_id] = is_following
			# Refresh UI to update follow buttons
			_update_preset_list()
		return
	
	presets_data = data.get("items", [])
	_update_preset_list()

func _update_preset_list():
	# Clear existing items
	for child in preset_list.get_children():
		child.queue_free()
	
	# Create preset items
	for i in range(presets_data.size()):
		var preset = presets_data[i]
		var item = _create_preset_item(preset, i)
		preset_list.add_child(item)

func _create_preset_item(preset: Dictionary, index: int) -> Control:
	var container = PanelContainer.new()
	container.custom_minimum_size = Vector2(0, 80)
	container.add_theme_stylebox_override("panel", _get_item_stylebox())
	
	var hbox = HBoxContainer.new()
	container.add_child(hbox)
	
	# Preset info
	var info = VBoxContainer.new()
	info.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	
	var name_label = Label.new()
	name_label.text = preset.get("name", "Untitled")
	name_label.add_theme_font_size_override("font_size", 16)
	name_label.add_theme_color_override("font_color", Color(0.9, 0.9, 0.9))
	hbox.add_child(name_label)
	
	var author_id = preset.get("author_id", "")
	var author_name = preset.get("author_name", preset.get("author_username", "Unknown"))
	var author_label = Label.new()
	author_label.text = "by " + author_name
	author_label.add_theme_font_size_override("font_size", 12)
	author_label.add_theme_color_override("font_color", Color(0.6, 0.6, 0.6))
	hbox.add_child(author_label)
	
	var category_label = Label.new()
	category_label.text = "🎵 " + preset.get("category", "Other") + \
	                       " | ⬇️ " + str(preset.get("downloads_count", 0)) + \
	                       " | ⭐ " + str(preset.get("rating", 0.0))
	category_label.add_theme_font_size_override("font_size", 11)
	category_label.add_theme_color_override("font_color", Color(0.5, 0.5, 0.5))
	hbox.add_child(category_label)
	
	# Buttons container
	var button_container = VBoxContainer.new()
	button_container.size_flags_horizontal = Control.SIZE_SHRINK_END
	button_container.alignment = BoxContainer.ALIGNMENT_CENTER
	
	# Download button
	var download_btn = Button.new()
	download_btn.text = "Download"
	download_btn.custom_minimum_size = Vector2(80, 30)
	download_btn.pressed.connect(_on_download_preset.bind(index))
	button_container.add_child(download_btn)
	
	# Share button (popup menu)
	var share_btn = Button.new()
	share_btn.text = "Share"
	share_btn.custom_minimum_size = Vector2(80, 24)
	share_btn.pressed.connect(_on_share_preset.bind(index, share_btn))
	button_container.add_child(share_btn)
	
	# Follow button (only if author_id exists and not self)
	if author_id and http.is_logged_in() and author_id != http.user_id:
		var follow_btn = Button.new()
		follow_btn.custom_minimum_size = Vector2(80, 24)
		follow_btn.pressed.connect(_on_follow_author.bind(author_id, follow_btn))
		button_container.add_child(follow_btn)
		
		# Check follow status
		if follow_states.has(author_id):
			follow_btn.text = "Following" if follow_states[author_id] else "Follow"
		else:
			follow_btn.text = "Follow"
			# Fetch follow status
			http.check_follow_status(author_id)
	
	hbox.add_child(button_container)
	
	return container

func _on_download_preset(index: int):
	if index >= presets_data.size():
		return
	
	var preset = presets_data[index]
	var preset_id = preset.get("id")
	
	if http.is_logged_in():
		# Track download
		_show_message("Downloading: " + preset.get("name", ""))
		# Download preset data
		http.download_preset(preset_id)
	else:
		_show_error("Please login to download presets")

func _on_follow_author(author_id: String, button: Button):
	if not http.is_logged_in():
		_show_error("Please login to follow creators")
		return
	
	# Toggle follow state
	var is_following = follow_states.get(author_id, false)
	
	if is_following:
		http.unfollow_user(author_id)
		button.text = "Follow"
	else:
		http.follow_user(author_id)
		button.text = "Following"
	
	# Toggle state (will be confirmed by server response)
	follow_states[author_id] = not is_following

func _on_share_preset(index: int, button: Button):
	if index >= presets_data.size():
		return
	
	var preset = presets_data[index]
	var preset_id = preset.get("id")
	
	# Create share popup
	var popup = PopupMenu.new()
	popup.add_item("Copy Link", "copy")
	popup.add_item("Share on Twitter", "twitter")
	popup.add_item("Share on Facebook", "facebook")
	popup.add_item("Share on Reddit", "reddit")
	popup.add_separator()
	popup.add_item("Close", "close")
	
	popup.position = button.get_global_position() + Vector2(0, button.size.y)
	popup.id_pressed.connect(_on_share_option_selected.bind(preset_id))
	add_child(popup)
	popup.popup()
	
	# Store popup reference to clean up later
	popup.about_to_popup.connect(func(): await get_tree().create_timer(2.0).timeout; popup.queue_free())

func _on_share_option_selected(id: Variant, preset_id: String):
	if typeof(id) == TYPE_STRING:
		id = int(id)
	
	match id:
		0:  # Copy
			http.share_to_social(preset_id, "copy")
			_show_message("Link copied to clipboard!")
		1:  # Twitter
			http.share_to_social(preset_id, "twitter")
		2:  # Facebook
			http.share_to_social(preset_id, "facebook")
		3:  # Reddit
			http.share_to_social(preset_id, "reddit")

func _get_item_stylebox() -> StyleBoxFlat:
	var style = StyleBoxFlat.new()
	style.bg_color = Color(0.15, 0.15, 0.18, 1.0)
	style.border_color = Color(0.3, 0.3, 0.35, 1.0)
	style.set_border_width_all(1)
	style.set_corner_radius_all(4)
	return style

func _show_error(message: String):
	# Show error message (could be a popup or label)
	print("Error: ", message)

func _show_message(message: String):
	# Show success message
	print("Message: ", message)
