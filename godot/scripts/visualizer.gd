extends Control

# WAVELET Real-time Waveform Visualizer
# Displays audio waveform in real-time

var sample_buffer: PackedVector2Array = PackedVector2Array()
var buffer_size: int = 512
var sample_rate: float = 48000.0

# Visual settings
var line_color: Color = Color(0.3, 0.7, 0.9, 1.0)
var background_color: Color = Color(0.1, 0.1, 0.15, 0.8)
var grid_color: Color = Color(0.2, 0.2, 0.3, 0.5)
var line_width: float = 2.0

# Animation
var time_elapsed: float = 0.0
var glow_intensity: float = 0.0

func _ready() -> void:
	# Initialize buffer
	sample_buffer.resize(buffer_size)
	sample_buffer.fill(Vector2.ZERO)
	
	# Connect to audio system
	_setup_audio_connection()

func _setup_audio_connection() -> void:
	# Try to connect to main audio player for visualization
	var main = get_parent()
	if main and main.has_method("get_audio_samples"):
		# Will be connected at runtime
		pass

func add_samples(samples: PackedVector2Array) -> void:
	# Add new samples to buffer
	for sample in samples:
		shift_buffer(sample)

func shift_buffer(new_sample: Vector2) -> void:
	# Shift buffer and add new sample
	for i in range(buffer_size - 1):
		sample_buffer[i] = sample_buffer[i + 1]
	sample_buffer[buffer_size - 1] = new_sample

func _process(delta: float) -> void:
	time_elapsed += delta
	
	# Pulse animation
	glow_intensity = 0.5 + 0.5 * sin(time_elapsed * 2.0)
	
	queue_redraw()

func _draw() -> void:
	var rect = get_rect()
	var center_y = rect.size.y / 2.0
	var amplitude = rect.size.y * 0.4
	
	# Draw background
	draw_rect(Rect2(Vector2.ZERO, rect.size), background_color)
	
	# Draw grid
	_draw_grid(rect, center_y)
	
	# Draw waveform
	if sample_buffer.size() >= 2:
		# Calculate points
		var points: PackedVector2Array = PackedVector2Array()
		var x_step = rect.size.x / float(buffer_size - 1)
		
		for i in range(sample_buffer.size()):
			var x = i * x_step
			var y = center_y - sample_buffer[i].y * amplitude
			y = clamp(y, 0.0, rect.size.y)
			points.append(Vector2(x, y))
		
		# Draw glow effect
		var glow_color = line_color
		glow_color.a = 0.3 * glow_intensity
		for width in range(4, 0, -1):
			draw_polyline(points, glow_color, width * line_width)
		
		# Draw main line
		draw_polyline(points, line_color, line_width)
		
		# Draw center line
		draw_line(Vector2(0, center_y), Vector2(rect.size.x, center_y), grid_color, 1.0)

func _draw_grid(rect: Rect2, center_y: float) -> void:
	# Draw horizontal grid lines
	var grid_spacing = rect.size.y / 8.0
	for i in range(1, 8):
		var y = i * grid_spacing
		draw_line(Vector2(0, y), Vector2(rect.size.x, y), grid_color, 0.5)
	
	# Draw vertical grid lines
	var x_spacing = rect.size.x / 16.0
	for i in range(1, 16):
		var x = i * x_spacing
		draw_line(Vector2(x, 0), Vector2(x, rect.size.y), grid_color, 0.5)

func set_line_color(color: Color) -> void:
	line_color = color

func set_background_color(color: Color) -> void:
	background_color = color
	queue_redraw()

func clear_buffer() -> void:
	sample_buffer.fill(Vector2.ZERO)
	queue_redraw()
