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
		_show_error(http.get_error_message(response))
		return
	
	var data = response.get("data", {})
	if typeof(data) != TYPE_DICTIONARY:
		_show_error("Invalid response format")
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
	
	var author_label = Label.new()
	author_label.text = "by " + preset.get("author_name", preset.get("author_username", "Unknown"))
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
	
	# Download button
	var download_btn = Button.new()
	download_btn.text = "Download"
	download_btn.size_flags_horizontal = Control.SIZE_SHRINK_END
	download_btn.pressed.connect(_on_download_preset.bind(index))
	hbox.add_child(download_btn)
	
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
