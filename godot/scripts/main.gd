extends Control

# WAVELET - Abstract Sound Synthesizer
# Main script for the synthesizer UI and audio processing

# References to UI elements
@onready var volume_slider: HSlider = $VBoxContainer/Controls/VolumeSlider
@onready var filter_slider: HSlider = $VBoxContainer/FilterSlider
@onready var resonance_slider: HSlider = $VBoxContainer/ResonanceSlider
@onready var audio_player: AudioStreamPlayer = $AudioStreamPlayer

# Audio stream for playback
var audio_stream: AudioStreamGenerator
var playback: AudioStreamGeneratorPlayback

# Synthesizer state
var volume: float = 0.7
var filter_cutoff: float = 2000.0
var filter_resonance: float = 1.0
var is_playing: bool = false

# MIDI note to frequency mapping
var note_frequencies: Dictionary = {}

# Currently held notes
var held_notes: Array[int] = []

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
	
	# Connect UI signals
	volume_slider.value_changed.connect(_on_volume_changed)
	filter_slider.value_changed.connect(_on_filter_changed)
	resonance_slider.value_changed.connect(_on_resonance_changed)
	
	# Connect preset buttons
	$VBoxContainer/PresetsGrid/PresetInit.pressed.connect(func(): load_preset("init"))
	$VBoxContainer/PresetsGrid/PresetBass.pressed.connect(func(): load_preset("bass"))
	$VBoxContainer/PresetsGrid/PresetPad.pressed.connect(func(): load_preset("pad"))
	$VBoxContainer/PresetsGrid/PresetLead.pressed.connect(func(): load_preset("lead"))
	
	# Initialize note frequencies
	initialize_note_frequencies()
	
	print("WAVELET Synthesizer ready!")

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

func _process(_delta: float) -> void:
	# Process audio buffer
	if playback:
		var frames_available = playback.get_frames_available()
		
		for i in range(frames_available):
			var left: float
			var right: float
			
			# Generate audio samples
			if held_notes.size() > 0:
				var sample: float = generate_sample()
				left = sample * volume
				right = sample * volume
			else:
				left = 0.0
				right = 0.0
			
			# Push samples to audio buffer
			playback.push_frame(Vector2(left, right))

func generate_sample() -> float:
	# Simple oscillator-based synthesis
	var sample: float = 0.0
	
	for note in held_notes:
		var freq = note_frequencies.get(note, 440.0)
		var phase = Time.get_ticks_msec() * 0.001 * freq * TAU
		var waveform = sin(phase)
		
		# Apply filter
		var filtered = apply_filter(waveform)
		sample += filtered
	
	# Average and normalize
	if held_notes.size() > 0:
		sample /= held_notes.size()
	
	return sample

func apply_filter(waveform: float) -> float:
	# Simple low-pass filter simulation
	var alpha = 1.0 / (1.0 + filter_resonance * 0.1)
	var cutoff_factor = clamp(filter_cutoff / 10000.0, 0.0, 1.0)
	
	var input_val = waveform
	var output_val = input_val * cutoff_factor + waveform * (1.0 - cutoff_factor) * alpha
	
	return output_val

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

func load_preset(preset_name: String) -> void:
	var preset_file = "presets/" + preset_name + ".json"
	
	if FileAccess.file_exists(preset_file):
		var file = FileAccess.open(preset_file, FileAccess.READ)
		var json = JSON.new()
		var error = json.parse(file.get_as_text())
		
		if error == OK:
			var data = json.get_data()
			apply_preset(data)
			print("Loaded preset: ", preset_name)
	else:
		print("Preset not found: ", preset_name)

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
