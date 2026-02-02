extends Control

# WAVELET - Abstract Sound Synthesizer
# Main script for the synthesizer UI and audio processing

# Community features
@onready var http_client: Node = $HTTPClient
@onready var community_panel: Control = $CommunityPanel
@onready var volume_slider: HSlider = $MainContainer/ControlsPanel/ControlsGrid/VolumeSlider
@onready var filter_slider: HSlider = $MainContainer/ControlsPanel/ControlsGrid/FilterSlider
@onready var resonance_slider: HSlider = $MainContainer/ControlsPanel/ControlsGrid/ResonanceSlider
@onready var attack_slider: HSlider = $MainContainer/ControlsPanel/ControlsGrid/AttackSlider
@onready var release_slider: HSlider = $MainContainer/ControlsPanel/ControlsGrid/ReleaseSlider
@onready var audio_player: AudioStreamPlayer = $AudioStreamPlayer
@onready var visualizer: Control = $WaveformVisualizer
@onready var theme_manager: Node = $ThemeManager

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

# MIDI note to frequency mapping
var note_frequencies: Dictionary = {}

# Currently held notes
var held_notes: Array[int] = []

# Active voices
var active_voices: Dictionary = {}  # note -> {phase: float, envelope: float}

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

func _ready() -> void:
	# Initialize audio system
	setup_audio()
	
	# Initialize community features
	_init_community()
	
	# Connect UI signals
	volume_slider.value_changed.connect(_on_volume_changed)
	filter_slider.value_changed.connect(_on_filter_changed)
	resonance_slider.value_changed.connect(_on_resonance_changed)
	attack_slider.value_changed.connect(_on_attack_changed)
	release_slider.value_changed.connect(_on_release_changed)
	
	# Connect theme buttons
	$MainContainer/TopBar/ThemeButtons/ThemeDark.pressed.connect(func(): _switch_theme(0))
	$MainContainer/TopBar/ThemeButtons/ThemeRetro.pressed.connect(func(): _switch_theme(1))
	$MainContainer/TopBar/ThemeButtons/ThemeCyber.pressed.connect(func(): _switch_theme(2))
	
	# Connect preset buttons
	$MainContainer/PresetsGrid/PresetInit.pressed.connect(func(): load_preset("init"))
	$MainContainer/PresetsGrid/PresetBass.pressed.connect(func(): load_preset("bass"))
	$MainContainer/PresetsGrid/PresetPad.pressed.connect(func(): load_preset("pad"))
	$MainContainer/PresetsGrid/PresetLead.pressed.connect(func(): load_preset("lead"))
	$MainContainer/PresetsGrid/PresetKeys.pressed.connect(func(): load_preset("keys"))
	$MainContainer/PresetsGrid/PresetStrings.pressed.connect(func(): load_preset("strings"))
	$MainContainer/PresetsGrid/PresetBell.pressed.connect(func(): load_preset("bell"))
	$MainContainer/PresetsGrid/PresetEffect.pressed.connect(func(): load_preset("effect"))
	
	# Connect share/export buttons
	$MainContainer/TopBar/ShareButtons/ExportButton.pressed.connect(_export_project)
	$MainContainer/TopBar/ShareButtons/ImportButton.pressed.connect(_import_project)
	$MainContainer/TopBar/ShareButtons/ShareCommunityButton.pressed.connect(_share_to_community)
	
	# Initialize note frequencies
	initialize_note_frequencies()
	
	print("🎮 WAVELET Synthesizer ready!")
	print("Theme system: Dark / Retro / Cyber")
	print("✓ Project sharing enabled: Export / Import / Share to Community")

func setup_audio() -> void:
	# Create audio stream generator
	audio_stream = AudioStreamGenerator.new()
	audio_stream.mix_rate = 48000
	audio_stream.resource_name = "WAVELET Output"
	
	# Set up audio stream player
	audio_player.stream = audio_stream
	audio_player.autoplay = true
	
	# Get playback interface
	playback = audio_player.get_stream_playback()

func _init_community() -> void:
	# Initialize HTTP client
	if http_client:
		print("🌐 Community features initialized")
		# Check if user is logged in
		if http_client.is_logged_in():
			print("✓ User is logged in")
	
	# Connect community panel signals if available
	if community_panel and community_panel.has_method("load_feed"):
		print("✓ Community panel ready")

func _process(delta: float) -> void:
	# Process audio buffer
	if playback:
		var frames_available = playback.get_frames_available()
		var samples: PackedVector2Array = PackedVector2Array()
		
		for i in range(frames_available):
			var left: float
			var right: float
			
			# Generate audio samples
			if held_notes.size() > 0:
				var sample: float = generate_sample(delta)
				left = sample * volume
				right = sample * volume
				
				# Collect samples for visualization
				samples.append(Vector2(left, right))
			else:
				left = 0.0
				right = 0.0
				samples.append(Vector2.ZERO)
			
			# Push samples to audio buffer
			playback.push_frame(Vector2(left, right))
		
		# Update visualizer
		if visualizer and samples.size() > 0:
			visualizer.add_samples(samples)

func generate_sample(delta: float) -> float:
	# Advanced synthesis with multiple oscillators and envelopes
	var sample: float = 0.0
	
	for note in held_notes:
		# Initialize voice if new
		if not active_voices.has(note):
			active_voices[note] = {
				"phase": 0.0,
				"envelope": 0.0,
				"attack_phase": 0.0
			}
		
		var voice = active_voices[note]
		var freq = note_frequencies.get(note, 440.0)
		
		# Phase accumulation
		voice["phase"] += freq * delta * TAU
		if voice["phase"] > TAU:
			voice["phase"] -= TAU
		
		# Generate oscillator waveform (sawtooth)
		var wave = 2.0 * (voice["phase"] / TAU) - 1.0
		
		# Envelope processing
		var envelope: float
		
		# Attack phase
		if voice["attack_phase"] < 1.0:
			voice["attack_phase"] += delta / max(attack_time, 0.001)
			voice["attack_phase"] = min(voice["attack_phase"], 1.0)
			envelope = voice["attack_phase"]
		else:
			# Sustain/Release
			envelope = 1.0
		
		voice["envelope"] = envelope
		
		# Apply filter
		var filtered = apply_filter(wave, note)
		
		sample += filtered * envelope
	
	# Average and normalize
	if held_notes.size() > 0:
		sample /= held_notes.size()
	
	return sample

func apply_filter(waveform: float, note: int) -> float:
	# Biquad low-pass filter simulation
	var cutoff_norm = clamp(filter_cutoff / 10000.0, 0.0, 1.0)
	var resonance_factor = clamp(filter_resonance / 10.0, 0.0, 1.0)
	
	# Calculate filter coefficients
	var alpha = sin(3.14159 * cutoff_norm) * (1.0 + resonance_factor * 2.0)
	var a0 = 1.0 + alpha
	var b0 = (1.0 - cos(3.14159 * cutoff_norm)) / 2.0
	var b1 = 1.0 - cos(3.14159 * cutoff_norm)
	var b2 = b0
	var a1 = -2.0 * cos(3.14159 * cutoff_norm)
	var a2 = 1.0 - alpha
	
	# Apply filter (simplified)
	var output = waveform * b0 / a0 + resonance_factor * 0.1
	output = clamp(output, -1.0, 1.0)
	
	return output

func initialize_note_frequencies() -> void:
	# Calculate frequencies for all MIDI notes
	for note in range(128):
		note_frequencies[note] = 440.0 * pow(2.0, (note - 69) / 12.0)

func _input(event: InputEvent) -> void:
	if event is InputEventKey:
		var key_event = event as InputEventKey
		
		if key_event.pressed:
			var note = key_to_note.get(key_event.keycode, -1)
			if note >= 0 and not held_notes.has(note):
				play_note(note)
		elif not key_event.pressed:
			var note = key_to_note.get(key_event.keycode, -1)
			if note >= 0:
				stop_note(note)

func play_note(note: int) -> void:
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

func _switch_theme(theme_index: int) -> void:
	if theme_manager and theme_manager.has_method("switch_theme"):
		theme_manager.switch_theme(theme_index)
		
		# Update visualizer colors based on theme
		if visualizer:
			match theme_index:
				0:  # Dark
					visualizer.set_line_color(Color(0.3, 0.7, 0.9, 1.0))
				1:  # Retro
					visualizer.set_line_color(Color(0.9, 0.6, 0.2, 1.0))
				2:  # Cyber
					visualizer.set_line_color(Color(0.0, 0.9, 0.8, 1.0))
		
		print("Switched to theme: ", ["Dark", "Retro", "Cyber"][theme_index])

func load_preset(preset_name: String) -> void:
	var preset_file = "presets/20_presets.json"
	
	if FileAccess.file_exists(preset_file):
		var file = FileAccess.open(preset_file, FileAccess.READ)
		var json = JSON.new()
		var error = json.parse(file.get_as_text())
		
		if error == OK:
			var data = json.get_data()
			if data.has("presets"):
				var found = false
				for preset in data["presets"]:
					if preset.has("name") and preset["name"] == preset_name:
						if preset.has("parameters"):
							apply_preset(preset["parameters"])
							print("🎹 Loaded preset: ", preset_name)
							found = true
							break
				
				if not found:
					print("Preset not found: ", preset_name)
			else:
				print("Invalid preset file format")
		else:
			print("Error parsing preset file")
	else:
		print("Preset file not found: ", preset_file)

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
	
	# Visual feedback
	if visualizer:
		visualizer.clear_buffer()

# ==========================================
# Project Sharing / Export-Import Functions
# ==========================================

func _export_project() -> void:
	"""
	Export current project as JSON file
	Users can save their patch and share it with others
	"""
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
			"release": release_time
		},
		"settings": {
			"theme": _get_current_theme(),
			"presets_loaded": []
		}
	}
	
	# Convert to JSON
	var json_string = JSON.stringify(project_data, "\t")
	
	# Generate filename with timestamp
	var timestamp = Time.get_unix_time_from_system()
	var filename = "wavelet_project_%d.wlp" % timestamp
	
	# Save to file
	var file = FileAccess.open(filename, FileAccess.WRITE)
	if file:
		file.store_string(json_string)
		print("📦 Project exported to: ", filename)
		_show_notification("Project exported to:\n" + filename)
	else:
		print("❌ Failed to export project")
		_show_notification("Failed to export project")

func _import_project() -> void:
	"""
	Import project from JSON file
	Load a shared patch into the synthesizer
	"""
	# For now, we'll simulate file picker
	# In a real implementation, use Godot's FileDialog
	var default_file = "presets/20_presets.json"
	
	if FileAccess.file_exists(default_file):
		var file = FileAccess.open(default_file, FileAccess.READ)
		var json = JSON.new()
		var error = json.parse(file.get_as_text())
		
		if error == OK:
			var data = json.get_data()
			if data.has("parameters"):
				apply_preset(data["parameters"])
				print("📥 Project imported successfully")
				_show_notification("Project imported!")
			else:
				print("⚠️ Invalid project file format")
				_show_notification("Invalid project file")
		else:
			print("❌ Failed to parse project file")
			_show_notification("Failed to parse file")
	else:
		print("📂 Please select a .wlp file to import")
		_show_notification("Select a .wlp file to import")

func _share_to_community() -> void:
	"""
	Share current project to community
	Upload to server and generate shareable link
	"""
	var project_data = {
		"name": "My WAVELET Patch",
		"description": "Created with WAVELET",
		"parameters": {
			"volume": volume,
			"filter_cutoff": filter_cutoff,
			"filter_resonance": filter_resonance,
			"attack": attack_time,
			"release": release_time
		},
		"is_public": true,
		"tags": ["patch", "wavelet"]
	}
	
	# Use HTTP client to upload
	if http_client and http_client.has_method("upload_preset"):
		print("🌐 Uploading project to community...")
		_show_notification("Sharing to community...")
		
		# Call HTTP client upload method (async)
		http_client.upload_preset(project_data)
	else:
		print("⚠️ Community features not available")
		_show_notification("Community features\nnot available")

func _get_current_theme() -> int:
	"""Get current theme index"""
	# This would be connected to the actual theme state
	return 0  # Default to Dark

func _show_notification(message: String) -> void:
	"""
	Show notification to user
	In a real implementation, use Godot's Popup or notification system
	"""
	print("🔔 Notification: ", message)
