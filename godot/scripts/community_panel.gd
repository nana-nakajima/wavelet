extends Control

# Community Panel - Browse and download community presets

@onready var http: WaveletHTTP = $"../HTTPClient"
@onready var preset_list: VBoxContainer = $Panel/ScrollContainer/PresetList
@onready var feed_type_option: OptionButton = $Panel/FeedControls/FeedType
@onready var search_input: LineEdit = $Panel/FeedControls/SearchInput
@onready var search_button: Button = $Panel/FeedControls/SearchButton
@onready var loading_label: Label = $Panel/LoadingLabel
@onready var login_panel: Panel = $LoginPanel
@onready var username_input: LineEdit = $LoginPanel/Form/Username
@onready var email_input: LineEdit = $LoginPanel/Form/Email
@onready var password_input: LineEdit = $LoginPanel/Form/Password
@onready var login_button: Button = $LoginPanel/Form/LoginButton
@onready var register_button: Button = $LoginPanel/Form/RegisterButton
@onready var logout_button: Button = $LoginPanel/LogoutButton
@onready var user_info_label: Label = $LoginPanel/UserInfo
@onready var prev_button: Button = $Panel/Pagination/PrevButton
@onready var next_button: Button = $Panel/Pagination/NextButton
@onready var page_label: Label = $Panel/Pagination/PageLabel

var current_page: int = 1
var current_feed_type: String = "latest"
var current_category: String = ""
var presets_data: Array = []
var follow_states: Dictionary = {}
var active_popups: Array = []

func _ready():
	# Wait for parent to be ready
	await get_tree().process_frame

	# Connect signals
	if http:
		http.request_completed.connect(_on_request_completed)
		http.auth_changed.connect(_on_auth_changed)

	# Feed type options
	feed_type_option.clear()
	feed_type_option.add_item("Latest", 0)
	feed_type_option.add_item("Popular", 1)
	feed_type_option.add_item("Featured", 2)
	feed_type_option.add_item("Trending", 3)
	feed_type_option.item_selected.connect(_on_feed_type_selected)

	# Connect buttons
	search_button.pressed.connect(_on_search_pressed)
	search_input.text_submitted.connect(func(_text): _on_search_pressed())
	login_button.pressed.connect(_on_login_pressed)
	register_button.pressed.connect(_on_register_pressed)
	logout_button.pressed.connect(_on_logout_pressed)
	prev_button.pressed.connect(_on_prev_page_pressed)
	next_button.pressed.connect(_on_next_page_pressed)

	# Update UI based on auth state
	if http:
		_on_auth_changed(http.is_logged_in(), {})

	# Load initial feed
	load_feed()

func _on_auth_changed(is_logged_in: bool, user_data: Dictionary):
	if not is_instance_valid(login_panel):
		return

	login_panel.visible = not is_logged_in
	logout_button.visible = is_logged_in

	if is_logged_in:
		user_info_label.text = "Logged in as: " + user_data.get("username", user_data.get("email", "User"))
		user_info_label.visible = true
	else:
		user_info_label.visible = false

func _on_login_pressed():
	if not http:
		return
	var email = email_input.text.strip_edges()
	var password = password_input.text
	if email and password:
		http.login(email, password)
		_show_message("Logging in...")

func _on_register_pressed():
	if not http:
		return
	var username = username_input.text.strip_edges()
	var email = email_input.text.strip_edges()
	var password = password_input.text
	if username and email and password:
		http.register(username, email, password)
		_show_message("Registering...")

func _on_logout_pressed():
	if http:
		http.logout()
		_show_message("Logged out")

func load_feed():
	if not http:
		return
	loading_label.visible = true
	preset_list.visible = false
	http.get_feed(current_feed_type, current_page, 20, current_category)

func search_presets():
	if not http:
		return
	var query = search_input.text.strip_edges()
	if query:
		loading_label.visible = true
		preset_list.visible = false
		http.search_presets(query, "", "newest", 1, 20)
	else:
		load_feed()

func _on_feed_type_selected(index: int):
	var types = ["latest", "popular", "featured", "trending"]
	current_feed_type = types[index] if index < types.size() else "latest"
	current_page = 1
	load_feed()

func _on_search_pressed():
	search_presets()

func _on_prev_page_pressed():
	if current_page > 1:
		current_page -= 1
		_update_page_label()
		load_feed()

func _on_next_page_pressed():
	current_page += 1
	_update_page_label()
	load_feed()

func _update_page_label():
	page_label.text = "Page " + str(current_page)

func _on_request_completed(response: Dictionary):
	loading_label.visible = false
	preset_list.visible = true

	if response.get("code") != 200:
		var endpoint = response.get("endpoint", "")
		if endpoint.ends_with("/follow/check"):
			return

		_show_error(http.get_error_message(response) if http else "Request failed")
		return

	var data = response.get("data", {})
	if typeof(data) != TYPE_DICTIONARY:
		if typeof(data) == TYPE_BOOL:
			return
		_show_error("Invalid response format")
		return

	var endpoint = response.get("endpoint", "")
	if endpoint.ends_with("/follow/check"):
		var is_following = data.get("following", false)
		var user_id = ""
		var parts = endpoint.split("/")
		if parts.size() >= 3:
			user_id = parts[2]
		if user_id:
			follow_states[user_id] = is_following
			_update_preset_list()
		return

	presets_data = data.get("items", [])
	_update_preset_list()

func _update_preset_list():
	for child in preset_list.get_children():
		child.queue_free()

	for i in range(presets_data.size()):
		var preset = presets_data[i]
		var item = _create_preset_item(preset, i)
		preset_list.add_child(item)

func _create_preset_item(preset: Dictionary, index: int) -> Control:
	var container = PanelContainer.new()
	container.custom_minimum_size = Vector2(0, 70)
	container.add_theme_stylebox_override("panel", _get_item_stylebox())

	var margin = MarginContainer.new()
	margin.add_theme_constant_override("margin_left", 10)
	margin.add_theme_constant_override("margin_right", 10)
	margin.add_theme_constant_override("margin_top", 8)
	margin.add_theme_constant_override("margin_bottom", 8)
	container.add_child(margin)

	var hbox = HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 10)
	margin.add_child(hbox)

	# Preset info
	var info = VBoxContainer.new()
	info.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	info.add_theme_constant_override("separation", 2)

	var name_label = Label.new()
	name_label.text = preset.get("name", "Untitled")
	name_label.add_theme_font_size_override("font_size", 14)
	name_label.add_theme_color_override("font_color", Color(0.95, 0.95, 0.95))
	info.add_child(name_label)

	var author_name = preset.get("author_name", preset.get("author_username", "Unknown"))
	var author_label = Label.new()
	author_label.text = "by " + author_name
	author_label.add_theme_font_size_override("font_size", 11)
	author_label.add_theme_color_override("font_color", Color(0.6, 0.6, 0.6))
	info.add_child(author_label)

	var stats_label = Label.new()
	stats_label.text = preset.get("category", "Other") + " | " + str(preset.get("downloads_count", 0)) + " downloads"
	stats_label.add_theme_font_size_override("font_size", 10)
	stats_label.add_theme_color_override("font_color", Color(0.5, 0.5, 0.5))
	info.add_child(stats_label)

	hbox.add_child(info)

	# Buttons container
	var button_container = VBoxContainer.new()
	button_container.size_flags_horizontal = Control.SIZE_SHRINK_END
	button_container.alignment = BoxContainer.ALIGNMENT_CENTER
	button_container.add_theme_constant_override("separation", 4)

	# Download button
	var download_btn = Button.new()
	download_btn.text = "Download"
	download_btn.custom_minimum_size = Vector2(80, 26)
	download_btn.pressed.connect(_on_download_preset.bind(index))
	button_container.add_child(download_btn)

	# Share button
	var share_btn = Button.new()
	share_btn.text = "Share"
	share_btn.custom_minimum_size = Vector2(80, 24)
	share_btn.pressed.connect(_on_share_preset.bind(index, share_btn))
	button_container.add_child(share_btn)

	hbox.add_child(button_container)

	return container

func _on_download_preset(index: int):
	if index >= presets_data.size():
		return

	var preset = presets_data[index]
	var preset_id = preset.get("id")

	if http and http.is_logged_in():
		_show_message("Downloading: " + preset.get("name", ""))
		http.download_preset(preset_id)
	else:
		_show_error("Please login to download")

func _on_share_preset(index: int, button: Button):
	if index >= presets_data.size():
		return

	var preset = presets_data[index]
	var preset_id = str(preset.get("id", ""))

	# Clean up old popups
	_cleanup_popups()

	# Create share popup
	var popup = PopupMenu.new()
	popup.add_item("Copy Link", 0)
	popup.add_item("Twitter", 1)
	popup.add_item("Facebook", 2)
	popup.add_item("Reddit", 3)

	popup.position = button.global_position + Vector2(0, button.size.y)
	popup.id_pressed.connect(_on_share_option_selected.bind(preset_id))
	popup.popup_hide.connect(_on_popup_closed.bind(popup))

	add_child(popup)
	active_popups.append(popup)
	popup.popup()

func _on_popup_closed(popup: PopupMenu):
	if popup in active_popups:
		active_popups.erase(popup)
	if is_instance_valid(popup):
		popup.queue_free()

func _cleanup_popups():
	for popup in active_popups:
		if is_instance_valid(popup):
			popup.queue_free()
	active_popups.clear()

func _on_share_option_selected(id: int, preset_id: String):
	if not http:
		return

	match id:
		0:  # Copy
			http.share_to_social(preset_id, "copy")
			_show_message("Link copied!")
		1:  # Twitter
			http.share_to_social(preset_id, "twitter")
		2:  # Facebook
			http.share_to_social(preset_id, "facebook")
		3:  # Reddit
			http.share_to_social(preset_id, "reddit")

func _get_item_stylebox() -> StyleBoxFlat:
	var style = StyleBoxFlat.new()
	style.bg_color = Color(0.12, 0.12, 0.15, 1.0)
	style.border_color = Color(0.25, 0.25, 0.3, 1.0)
	style.set_border_width_all(1)
	style.set_corner_radius_all(4)
	return style

func _show_error(message: String):
	_show_toast(message, Color(0.9, 0.3, 0.3))

func _show_message(message: String):
	_show_toast(message, Color(0.3, 0.7, 0.4))

func _show_toast(message: String, color: Color):
	# Create toast notification
	var toast = PanelContainer.new()
	var style = StyleBoxFlat.new()
	style.bg_color = Color(0.15, 0.15, 0.2, 0.95)
	style.border_color = color
	style.set_border_width_all(2)
	style.set_corner_radius_all(6)
	style.content_margin_left = 15
	style.content_margin_right = 15
	style.content_margin_top = 10
	style.content_margin_bottom = 10
	toast.add_theme_stylebox_override("panel", style)

	var label = Label.new()
	label.text = message
	label.add_theme_color_override("font_color", Color(0.9, 0.9, 0.9))
	toast.add_child(label)

	toast.position = Vector2(10, 10)
	add_child(toast)

	# Auto-remove after delay
	var tween = create_tween()
	tween.tween_interval(2.5)
	tween.tween_property(toast, "modulate:a", 0.0, 0.3)
	tween.tween_callback(toast.queue_free)
