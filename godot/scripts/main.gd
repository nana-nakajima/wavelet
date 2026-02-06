extends Control

# WAVELET - Abstract Sound Synthesizer
# Main script for the synthesizer UI and audio processing

# Node references - updated for new UI layout
@onready var http_client: Node = $HTTPClient
@onready var community_panel: Control = $MainHSplit/RightPanel/CommunityPanel
@onready var volume_slider: HSlider = $MainHSplit/LeftPanel/ControlsSection/ControlsPanel/ControlsGrid/VolumeSlider
@onready var filter_slider: HSlider = $MainHSplit/LeftPanel/ControlsSection/ControlsPanel/ControlsGrid/FilterSlider
@onready var resonance_slider: HSlider = $MainHSplit/LeftPanel/ControlsSection/ControlsPanel/ControlsGrid/ResonanceSlider
@onready var attack_slider: HSlider = $MainHSplit/LeftPanel/ControlsSection/ControlsPanel/ControlsGrid/AttackSlider
@onready var release_slider: HSlider = $MainHSplit/LeftPanel/ControlsSection/ControlsPanel/ControlsGrid/ReleaseSlider
@onready var waveform_option: OptionButton = $MainHSplit/LeftPanel/ControlsSection/ControlsPanel/ControlsGrid/WaveformOption
@onready var category_filter: OptionButton = $MainHSplit/LeftPanel/PresetsSection/PresetsHeader/CategoryFilter
@onready var audio_player: AudioStreamPlayer = $AudioStreamPlayer
@onready var visualizer: Control = $MainHSplit/LeftPanel/WaveformVisualizer
@onready var theme_manager: Node = $ThemeManager
@onready var notification_panel: PanelContainer = $NotificationPanel
@onready var notification_label: Label = $NotificationPanel/NotificationLabel

# Audio stream for playback
var audio_stream: AudioStreamGenerator
var playback: AudioStreamGeneratorPlayback

# Synthesizer state
var volume: float = 0.7
var filter_cutoff: float = 2000.0
var filter_resonance: float = 1.0
var attack_time: float = 0.01
var release_time: float = 0.5
var is_playing: bool = false
var current_waveform: int = 0  # 0=saw, 1=sine, 2=square, 3=triangle

# Waveform types
enum Waveform { SAWTOOTH, SINE, SQUARE, TRIANGLE }

# MIDI note to frequency mapping
var note_frequencies: Dictionary = {}

# Currently held notes
var held_notes: Array[int] = []

# Active voices
var active_voices: Dictionary = {}  # note -> {phase: float, envelope: float, state: String}

# Keyboard mapping (English layout)
var key_to_note: Dictionary = {
	KEY_A: 60,  # C4
	KEY_W: 61,  # C#4
	KEY_S: 62,  # D4
	KEY_E: 63,  # D#4
	KEY_D: 64,  # E4
	KEY_F: 65,  # F4
	KEY_T: 66,  # F#4
	KEY_G: 67,  # G4
	KEY_Y: 68,  # G#4
	KEY_H: 69,  # A4
	KEY_U: 70,  # A#4
	KEY_J: 71,  # B4
	KEY_K: 72,  # C5
	KEY_O: 73,  # C#5
	KEY_L: 74,  # D5
	KEY_P: 75,  # D#5
	KEY_SEMICOLON: 76,  # E5
}

# Virtual keyboard button to note mapping
var keyboard_buttons: Dictionary = {}

# Notification timer
var notification_timer: float = 0.0

func _ready() -> void:
	# Initialize audio system
	setup_audio()

	# Initialize waveform options
	_setup_waveform_options()

	# Initialize category filter
	_setup_category_filter()

	# Initialize community features
	_init_community()

	# Connect UI signals
	volume_slider.value_changed.connect(_on_volume_changed)
	filter_slider.value_changed.connect(_on_filter_changed)
	resonance_slider.value_changed.connect(_on_resonance_changed)
	attack_slider.value_changed.connect(_on_attack_changed)
	release_slider.value_changed.connect(_on_release_changed)
	waveform_option.item_selected.connect(_on_waveform_changed)
	category_filter.item_selected.connect(_on_category_changed)

	# Connect theme buttons
	var theme_buttons = $MainHSplit/LeftPanel/Header/ThemeButtons
	theme_buttons.get_node("ThemeDark").pressed.connect(func(): _switch_theme(0))
	theme_buttons.get_node("ThemeRetro").pressed.connect(func(): _switch_theme(1))
	theme_buttons.get_node("ThemeCyber").pressed.connect(func(): _switch_theme(2))

	# Connect share buttons
	var share_buttons = $MainHSplit/LeftPanel/Header/ShareButtons
	share_buttons.get_node("ExportButton").pressed.connect(_export_project)
	share_buttons.get_node("ImportButton").pressed.connect(_import_project)
	share_buttons.get_node("ShareCommunityButton").pressed.connect(_share_to_community)

	# Connect preset buttons
	_connect_preset_buttons()

	# Connect virtual keyboard buttons
	_connect_keyboard_buttons()

	# Initialize note frequencies
	initialize_note_frequencies()

	print("WAVELET Synthesizer ready")

func _setup_waveform_options() -> void:
	waveform_option.clear()
	waveform_option.add_item("Sawtooth", Waveform.SAWTOOTH)
	waveform_option.add_item("Sine", Waveform.SINE)
	waveform_option.add_item("Square", Waveform.SQUARE)
	waveform_option.add_item("Triangle", Waveform.TRIANGLE)

func _setup_category_filter() -> void:
	category_filter.clear()
	category_filter.add_item("All Categories")
	category_filter.add_item("Bass")
	category_filter.add_item("Pad")
	category_filter.add_item("Lead")
	category_filter.add_item("Keys")
	category_filter.add_item("Strings")
	category_filter.add_item("Bell")
	category_filter.add_item("Effect")

func _connect_preset_buttons() -> void:
	var presets_grid = $MainHSplit/LeftPanel/PresetsSection/PresetsScroll/PresetsGrid
	presets_grid.get_node("PresetInit").pressed.connect(func(): load_preset("init"))
	presets_grid.get_node("PresetBass").pressed.connect(func(): load_preset("bass"))
	presets_grid.get_node("PresetPad").pressed.connect(func(): load_preset("pad"))
	presets_grid.get_node("PresetLead").pressed.connect(func(): load_preset("lead"))
	presets_grid.get_node("PresetKeys").pressed.connect(func(): load_preset("electric_piano"))
	presets_grid.get_node("PresetStrings").pressed.connect(func(): load_preset("strings"))
	presets_grid.get_node("PresetBell").pressed.connect(func(): load_preset("synth_bell"))
	presets_grid.get_node("PresetEffect").pressed.connect(func(): load_preset("sweep"))
	presets_grid.get_node("PresetAcid").pressed.connect(func(): load_preset("acid"))
	presets_grid.get_node("PresetPluck").pressed.connect(func(): load_preset("pluck"))
	presets_grid.get_node("PresetBrass").pressed.connect(func(): load_preset("brass"))
	presets_grid.get_node("PresetSub").pressed.connect(func(): load_preset("sub"))

func _connect_keyboard_buttons() -> void:
	var keyboard = $MainHSplit/LeftPanel/KeyboardSection/KeyboardContainer
	var button_notes = {
		"KeyC": 60, "KeyCS": 61, "KeyD": 62, "KeyDS": 63, "KeyE": 64,
		"KeyF": 65, "KeyFS": 66, "KeyG": 67, "KeyGS": 68, "KeyA": 69,
		"KeyAS": 70, "KeyB": 71, "KeyC2": 72, "KeyCS2": 73, "KeyD2": 74,
		"KeyDS2": 75, "KeyE2": 76
	}

	for button_name in button_notes:
		var button = keyboard.get_node(button_name)
		var note = button_notes[button_name]
		keyboard_buttons[button] = note
		button.button_down.connect(_on_keyboard_button_down.bind(note))
		button.button_up.connect(_on_keyboard_button_up.bind(note))

func _on_keyboard_button_down(note: int) -> void:
	if not held_notes.has(note):
		play_note(note)

func _on_keyboard_button_up(note: int) -> void:
	stop_note(note)

func setup_audio() -> void:
	audio_stream = AudioStreamGenerator.new()
	audio_stream.mix_rate = 48000
	audio_stream.buffer_length = 0.1

	audio_player.stream = audio_stream
	audio_player.play()

	playback = audio_player.get_stream_playback()

func _init_community() -> void:
	if http_client:
		print("Community features initialized")
		if http_client.is_logged_in():
			print("User is logged in")

	if community_panel and community_panel.has_method("load_feed"):
		print("Community panel ready")

func _process(delta: float) -> void:
	# Handle notification fade
	if notification_panel.visible:
		notification_timer -= delta
		if notification_timer <= 0:
			notification_panel.visible = false

	# Process audio buffer
	if playback:
		var frames_available = playback.get_frames_available()
		var samples: PackedVector2Array = PackedVector2Array()
		var sample_delta = 1.0 / 48000.0

		for i in range(frames_available):
			var sample: float = 0.0

			if held_notes.size() > 0 or _has_releasing_voices():
				sample = generate_sample(sample_delta)

			var output = sample * volume
			output = clamp(output, -1.0, 1.0)
			samples.append(Vector2(output, output))
			playback.push_frame(Vector2(output, output))

		if visualizer and samples.size() > 0:
			visualizer.add_samples(samples)

func _has_releasing_voices() -> bool:
	for note in active_voices:
		if active_voices[note]["state"] == "release" and active_voices[note]["envelope"] > 0.001:
			return true
	return false

func generate_sample(delta: float) -> float:
	var sample: float = 0.0
	var voice_count: int = 0
	var voices_to_remove: Array = []

	# Process held notes
	for note in held_notes:
		if not active_voices.has(note):
			active_voices[note] = {
				"phase": 0.0,
				"envelope": 0.0,
				"state": "attack"
			}

		var voice = active_voices[note]
		voice["state"] = "attack" if voice["envelope"] < 1.0 else "sustain"
		sample += _process_voice(voice, note, delta)
		voice_count += 1

	# Process releasing voices
	for note in active_voices:
		if not held_notes.has(note):
			var voice = active_voices[note]
			if voice["state"] != "release":
				voice["state"] = "release"

			if voice["envelope"] > 0.001:
				sample += _process_voice(voice, note, delta)
				voice_count += 1
			else:
				voices_to_remove.append(note)

	# Clean up finished voices
	for note in voices_to_remove:
		active_voices.erase(note)

	if voice_count > 0:
		sample /= sqrt(float(voice_count))  # Better mixing than simple average

	return sample

func _process_voice(voice: Dictionary, note: int, delta: float) -> float:
	var freq = note_frequencies.get(note, 440.0)

	# Phase accumulation
	voice["phase"] += freq * delta * TAU
	if voice["phase"] > TAU:
		voice["phase"] -= TAU

	# Generate waveform
	var wave = _generate_waveform(voice["phase"])

	# Envelope processing
	match voice["state"]:
		"attack":
			voice["envelope"] += delta / max(attack_time, 0.001)
			voice["envelope"] = min(voice["envelope"], 1.0)
		"sustain":
			voice["envelope"] = 1.0
		"release":
			voice["envelope"] -= delta / max(release_time, 0.01)
			voice["envelope"] = max(voice["envelope"], 0.0)

	# Apply filter
	var filtered = apply_filter(wave, note)

	return filtered * voice["envelope"]

func _generate_waveform(phase: float) -> float:
	match current_waveform:
		Waveform.SAWTOOTH:
			return 2.0 * (phase / TAU) - 1.0
		Waveform.SINE:
			return sin(phase)
		Waveform.SQUARE:
			return 1.0 if phase < PI else -1.0
		Waveform.TRIANGLE:
			var t = phase / TAU
			return 4.0 * abs(t - 0.5) - 1.0
		_:
			return 2.0 * (phase / TAU) - 1.0

func apply_filter(waveform: float, note: int) -> float:
	var cutoff_norm = clamp(filter_cutoff / 10000.0, 0.01, 1.0)
	var resonance_factor = clamp(filter_resonance / 10.0, 0.0, 0.99)

	# Simple one-pole lowpass approximation
	var rc = 1.0 / (cutoff_norm * TAU)
	var alpha = 1.0 / (1.0 + rc)

	var output = waveform * alpha + resonance_factor * waveform * 0.2
	return clamp(output, -1.0, 1.0)

func initialize_note_frequencies() -> void:
	for note in range(128):
		note_frequencies[note] = 440.0 * pow(2.0, (note - 69) / 12.0)

func _input(event: InputEvent) -> void:
	if event is InputEventKey:
		var key_event = event as InputEventKey

		if key_event.pressed and not key_event.echo:
			var note = key_to_note.get(key_event.keycode, -1)
			if note >= 0 and not held_notes.has(note):
				play_note(note)
		elif not key_event.pressed:
			var note = key_to_note.get(key_event.keycode, -1)
			if note >= 0:
				stop_note(note)

func play_note(note: int) -> void:
	if not held_notes.has(note):
		held_notes.append(note)
	is_playing = true

func stop_note(note: int) -> void:
	held_notes.erase(note)
	if held_notes.is_empty():
		is_playing = false

func _on_volume_changed(value: float) -> void:
	volume = value

func _on_filter_changed(value: float) -> void:
	filter_cutoff = value

func _on_resonance_changed(value: float) -> void:
	filter_resonance = value

func _on_attack_changed(value: float) -> void:
	attack_time = value

func _on_release_changed(value: float) -> void:
	release_time = value

func _on_waveform_changed(index: int) -> void:
	current_waveform = index

func _on_category_changed(index: int) -> void:
	# Filter presets by category (future implementation)
	pass

func _switch_theme(theme_index: int) -> void:
	if theme_manager and theme_manager.has_method("switch_theme"):
		theme_manager.switch_theme(theme_index)

		if visualizer:
			match theme_index:
				0:  # Dark
					visualizer.set_line_color(Color(0.3, 0.7, 0.9, 1.0))
				1:  # Retro
					visualizer.set_line_color(Color(0.9, 0.6, 0.2, 1.0))
				2:  # Cyber
					visualizer.set_line_color(Color(0.0, 0.9, 0.8, 1.0))

func load_preset(preset_name: String) -> void:
	var preset_file = "res://presets/20_presets.json"

	if FileAccess.file_exists(preset_file):
		var file = FileAccess.open(preset_file, FileAccess.READ)
		var json = JSON.new()
		var error = json.parse(file.get_as_text())

		if error == OK:
			var data = json.get_data()
			if data.has("presets"):
				for preset in data["presets"]:
					if preset.has("name") and preset["name"] == preset_name:
						if preset.has("parameters"):
							apply_preset(preset["parameters"])
							_show_notification("Loaded: " + preset_name)
							return

				_show_notification("Preset not found: " + preset_name)
		else:
			_show_notification("Error loading presets")
	else:
		_show_notification("Preset file not found")

func apply_preset(preset: Dictionary) -> void:
	if preset.has("volume"):
		volume = preset["volume"]
		volume_slider.value = volume

	if preset.has("filter_cutoff"):
		filter_cutoff = preset["filter_cutoff"]
		filter_slider.value = filter_cutoff

	if preset.has("filter_resonance"):
		filter_resonance = preset["filter_resonance"]
		resonance_slider.value = filter_resonance

	if preset.has("attack"):
		attack_time = preset["attack"]
		attack_slider.value = attack_time

	if preset.has("release"):
		release_time = preset["release"]
		release_slider.value = release_time

	if preset.has("waveform"):
		var waveform_name = preset["waveform"]
		match waveform_name:
			"sawtooth":
				current_waveform = Waveform.SAWTOOTH
				waveform_option.select(0)
			"sine":
				current_waveform = Waveform.SINE
				waveform_option.select(1)
			"square":
				current_waveform = Waveform.SQUARE
				waveform_option.select(2)
			"triangle":
				current_waveform = Waveform.TRIANGLE
				waveform_option.select(3)

	if visualizer:
		visualizer.clear_buffer()

func _export_project() -> void:
	var project_data = {
		"version": "2.3.0",
		"name": "WAVELET Project",
		"created_at": Time.get_datetime_string_from_system(true),
		"author": "Unknown",
		"parameters": {
			"volume": volume,
			"filter_cutoff": filter_cutoff,
			"filter_resonance": filter_resonance,
			"attack": attack_time,
			"release": release_time,
			"waveform": ["sawtooth", "sine", "square", "triangle"][current_waveform]
		},
		"settings": {
			"theme": _get_current_theme()
		}
	}

	var json_string = JSON.stringify(project_data, "\t")
	var timestamp = Time.get_unix_time_from_system()
	var filename = "user://wavelet_project_%d.wlp" % timestamp

	var file = FileAccess.open(filename, FileAccess.WRITE)
	if file:
		file.store_string(json_string)
		_show_notification("Project exported")
	else:
		_show_notification("Export failed")

func _import_project() -> void:
	# Open file dialog for import
	var dialog = FileDialog.new()
	dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	dialog.access = FileDialog.ACCESS_FILESYSTEM
	dialog.filters = PackedStringArray(["*.wlp ; WAVELET Project", "*.json ; JSON File"])
	dialog.file_selected.connect(_on_import_file_selected)
	add_child(dialog)
	dialog.popup_centered(Vector2(600, 400))

func _on_import_file_selected(path: String) -> void:
	if FileAccess.file_exists(path):
		var file = FileAccess.open(path, FileAccess.READ)
		var json = JSON.new()
		var error = json.parse(file.get_as_text())

		if error == OK:
			var data = json.get_data()
			if data.has("parameters"):
				apply_preset(data["parameters"])
				_show_notification("Project imported")
			else:
				_show_notification("Invalid project file")
		else:
			_show_notification("Failed to parse file")
	else:
		_show_notification("File not found")

func _share_to_community() -> void:
	if not http_client or not http_client.is_logged_in():
		_show_notification("Please login to share")
		return

	var project_data = {
		"name": "My WAVELET Patch",
		"description": "Created with WAVELET",
		"parameters": {
			"volume": volume,
			"filter_cutoff": filter_cutoff,
			"filter_resonance": filter_resonance,
			"attack": attack_time,
			"release": release_time,
			"waveform": ["sawtooth", "sine", "square", "triangle"][current_waveform]
		},
		"is_public": true,
		"tags": ["patch", "wavelet"]
	}

	if http_client.has_method("upload_preset"):
		http_client.upload_preset(project_data)
		_show_notification("Sharing to community...")
	else:
		_show_notification("Share not available")

func _get_current_theme() -> int:
	if theme_manager and theme_manager.has("current_theme"):
		return theme_manager.current_theme
	return 0

func _show_notification(message: String) -> void:
	notification_label.text = message
	notification_panel.visible = true
	notification_timer = 3.0
