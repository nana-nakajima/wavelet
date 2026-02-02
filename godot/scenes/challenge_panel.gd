extends Control

# UI Components
var title_label: Label
var description_label: Label
var theme_label: Label
var timer_label: Label
var submissions_list: VBoxContainer
var submit_button: Button
var refresh_button: Button

# Data
var current_challenge: Dictionary = {}
var jwt_token: String = ""
var http_client: HTTPRequest

func _ready():
    # Setup UI layout
    _setup_ui()
    
    # Add HTTP request node
    http_client = HTTPRequest.new()
    add_child(http_client)
    http_client.request_completed.connect(_on_request_completed)
    
    # Load JWT token
    _load_jwt_token()
    
    # Fetch active challenges
    fetch_challenges()

func _setup_ui():
    # Main container
    var main_container = VBoxContainer.new()
    main_container.set_anchors_preset(Control.PRESET_FULL_RECT)
    main_container.add_theme_constant_override("separation", 20)
    add_child(main_container)
    
    # Header
    var header = HBoxContainer.new()
    header.add_theme_constant_override("separation", 10)
    main_container.add_child(header)
    
    # Title
    title_label = Label.new()
    title_label.text = "🎮 Weekly Challenge"
    title_label.add_theme_font_size_override("font_size", 32)
    title_label.add_theme_color_override("font_color", Color(0.2, 0.8, 1.0))
    header.add_child(title_label)
    
    # Refresh button
    refresh_button = Button.new()
    refresh_button.text = "🔄 Refresh"
    refresh_button.pressed.connect(fetch_challenges)
    header.add_child(refresh_button)
    
    # Timer
    timer_label = Label.new()
    timer_label.text = "⏰ Loading..."
    timer_label.add_theme_font_size_override("font_size", 20)
    timer_label.add_theme_color_override("font_color", Color(1.0, 0.8, 0.2))
    header.add_child(timer_label)
    
    # Theme badge
    theme_label = Label.new()
    theme_label.text = "Theme: Loading..."
    theme_label.add_theme_font_size_override("font_size", 24)
    theme_label.add_theme_color_override("font_color", Color(0.8, 0.4, 1.0))
    main_container.add_child(theme_label)
    
    # Description
    description_label = Label.new()
    description_label.text = "Loading challenge details..."
    description_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
    description_label.custom_minimum.y = 60
    main_container.add_child(description_label)
    
    # Divider
    var divider = HSeparator.new()
    main_container.add_child(divider)
    
    # Submissions section
    var submissions_title = Label.new()
    submissions_title.text = "📝 Submissions"
    submissions_title.add_theme_font_size_override("font_size", 24)
    submissions_title.add_theme_color_override("font_color", Color(0.4, 1.0, 0.6))
    main_container.add_child(submissions_title)
    
    # Scroll container for submissions
    var scroll = ScrollContainer.new()
    scroll.custom_minimum.y = 300
    scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
    main_container.add_child(scroll)
    
    submissions_list = VBoxContainer.new()
    submissions_list.add_theme_constant_override("separation", 10)
    submissions_list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    scroll.add_child(submissions_list)
    
    # Submit button
    submit_button = Button.new()
    submit_button.text = "🚀 Submit Your Project"
    submit_button.custom_minimum.x = 200
    submit_button.pressed.connect(_on_submit_pressed)
    main_container.add_child(submit_button)

func _load_jwt_token():
    var file = FileAccess.open("user_session.json", FileAccess.READ)
    if file:
        var json = JSON.new()
        var error = json.parse(file.get_as_text())
        if error == OK:
            var data = json.get_data()
            if data.has("jwt"):
                jwt_token = data.jwt

func fetch_challenges():
    var url = "http://127.0.0.1:8080/api/challenges"
    var headers = ["Content-Type: application/json"]
    
    var error = http_client.request(url, headers, HTTPClient.METHOD_GET, "")
    if error != OK:
        print("Failed to fetch challenges")

func _on_request_completed(result, response_code, headers, body):
    if response_code == 200:
        var json = JSON.new()
        var error = json.parse(body.get_string_from_utf8())
        if error == OK:
            var challenges = json.get_data()
            if challenges.size() > 0:
                current_challenge = challenges[0]
                _update_challenge_display()

func _update_challenge_display():
    if current_challenge.is_empty():
        return
    
    # Update labels
    title_label.text = "🎮 " + current_challenge.get("title", "Weekly Challenge")
    theme_label.text = "🎨 Theme: " + current_challenge.get("theme", "Free Style")
    description_label.text = current_challenge.get("description", "No description")
    
    # Update timer
    var end_date = current_challenge.get("end_date", "")
    if end_date != "":
        _update_timer(end_date)
    
    # Update participant count
    var count = current_challenge.get("participant_count", 0)
    timer_label.text = "👥 " + str(count) + " participants | ⏰ "
    
    # Fetch submissions
    fetch_submissions()
    
    # For demo - create sample submissions
    await get_tree().create_timer(0.5).timeout
    _create_sample_submissions()

func _update_timer(end_date_str: String):
    # Parse ISO date and calculate countdown
    var end_date = Time.get_unix_time_from_datetime_string(end_date_str)
    var now = Time.get_unix_time_from_system()
    var remaining = end_date - now
    
    if remaining > 0:
        var days = int(remaining / 86400)
        var hours = int((remaining % 86400) / 3600)
        var minutes = int((remaining % 3600) / 60)
        timer_label.text = "⏰ " + str(days) + "d " + str(hours) + "h " + str(minutes) + "m remaining"
    else:
        timer_label.text = "⏰ Challenge Ended!"

func fetch_submissions():
    if current_challenge.is_empty():
        return
    
    var challenge_id = current_challenge.get("id", 0)
    var url = "http://127.0.0.1:8080/api/challenges/" + str(challenge_id)
    var headers = ["Content-Type: application/json"]
    
    var error = http_client.request(url, headers, HTTPClient.METHOD_GET, "")
    if error != OK:
        print("Failed to fetch submissions")

func _on_submit_pressed():
    # Open project export dialog
    print("Opening project export dialog...")
    # TODO: Implement project export and submission

# For demo purposes - create a sample challenge
func create_sample_challenge():
    var sample_challenge = {
        "title": "Retro Synth Wave Challenge",
        "description": "Create a synthwave track using only WAVELET presets! Show us your 80s vibes.",
        "theme": "80s Retro",
        "start_date": Time.get_iso_datetime_string_from_system(),
        "end_date": "2026-02-10T23:59:59Z",
        "status": "active",
        "participant_count": 0
    }
    
    current_challenge = sample_challenge
    _update_challenge_display()

func _create_sample_submissions():
    var sample_submissions = [
        {"username": "SynthMaster", "project_name": "Neon Nights", "votes": 42, "rank": 1, "description": "Pure 80s nostalgia with modern touch"},
        {"username": "RetroFan", "project_name": "Miami Sunset", "votes": 38, "rank": 2, "description": "Driving bassline and dreamy pads"},
        {"username": "WaveRider", "project_name": "Cyber Dreams", "votes": 31, "rank": 3, "description": "Dark synthwave with aggressive leads"},
        {"username": "PixelBeat", "project_name": "Digital Love", "votes": 27, "rank": 4, "description": "Chiptune-inspired arpeggios"},
        {"username": "BassDrop", "project_name": "Night Drive", "votes": 24, "rank": 5, "description": "Heavy bass and atmospheric pads"}
    ]
    
    for sub in sample_submissions:
        _add_submission_to_list(sub)

func _add_submission_to_list(submission: Dictionary):
    var item = HBoxContainer.new()
    item.add_theme_constant_override("separation", 10)
    
    # Rank badge
    var rank_label = Label.new()
    var rank = submission.get("rank", 0)
    if rank <= 3:
        var medals = ["🥇", "🥈", "🥉"]
        rank_label.text = medals[rank - 1]
    else:
        rank_label.text = "#" + str(rank)
    rank_label.custom_minimum.x = 50
    rank_label.add_theme_font_size_override("font_size", 20)
    item.add_child(rank_label)
    
    # Project info
    var info_vbox = VBoxContainer.new()
    info_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    
    var name_label = Label.new()
    name_label.text = submission.get("project_name", "Untitled")
    name_label.add_theme_font_size_override("font_size", 18)
    name_label.add_theme_color_override("font_color", Color(0.2, 0.8, 1.0))
    info_vbox.add_child(name_label)
    
    var author_label = Label.new()
    author_label.text = "by @" + submission.get("username", "Unknown")
    author_label.add_theme_font_size_override("font_size", 14)
    author_label.add_theme_color_override("font_color", Color(0.6, 0.6, 0.6))
    info_vbox.add_child(author_label)
    
    var desc_label = Label.new()
    desc_label.text = submission.get("description", "")
    desc_label.add_theme_font_size_override("font_size", 12)
    desc_label.add_theme_color_override("font_color", Color(0.8, 0.8, 0.8))
    info_vbox.add_child(desc_label)
    
    item.add_child(info_vbox)
    
    # Vote count
    var votes_label = Label.new()
    votes_label.text = "👍 " + str(submission.get("votes", 0))
    votes_label.add_theme_font_size_override("font_size", 16)
    votes_label.add_theme_color_override("font_color", Color(1.0, 0.6, 0.2))
    votes_label.custom_minimum.x = 80
    item.add_child(votes_label)
    
    submissions_list.add_child(item)
