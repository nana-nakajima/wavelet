extends Node

# WAVELET Theme Manager
# Handles theme switching and color schemes

enum ThemeType { DARK, RETRO, CYBER }

var current_theme: ThemeType = ThemeType.DARK

# Theme color palettes
var themes: Dictionary = {
	ThemeType.DARK: {
		"background": Color(0.1, 0.1, 0.15, 1.0),
		"surface": Color(0.15, 0.15, 0.2, 1.0),
		"primary": Color(0.3, 0.5, 0.8, 1.0),
		"secondary": Color(0.8, 0.4, 0.5, 1.0),
		"accent": Color(0.4, 0.7, 0.5, 1.0),
		"text": Color(0.9, 0.9, 0.95, 1.0),
		"text_dim": Color(0.6, 0.6, 0.65, 1.0),
		"slider_track": Color(0.25, 0.25, 0.35, 1.0),
		"slider_fill": Color(0.3, 0.5, 0.8, 1.0),
		"button_normal": Color(0.2, 0.2, 0.25, 1.0),
		"button_hover": Color(0.25, 0.25, 0.3, 1.0),
		"button_pressed": Color(0.15, 0.15, 0.2, 1.0),
	},
	
	ThemeType.RETRO: {
		"background": Color(0.2, 0.1, 0.3, 1.0),
		"surface": Color(0.3, 0.15, 0.35, 1.0),
		"primary": Color(0.9, 0.6, 0.2, 1.0),
		"secondary": Color(0.4, 0.9, 0.6, 1.0),
		"accent": Color(0.9, 0.3, 0.5, 1.0),
		"text": Color(0.95, 0.9, 0.8, 1.0),
		"text_dim": Color(0.7, 0.65, 0.6, 1.0),
		"slider_track": Color(0.4, 0.25, 0.5, 1.0),
		"slider_fill": Color(0.9, 0.6, 0.2, 1.0),
		"button_normal": Color(0.35, 0.2, 0.4, 1.0),
		"button_hover": Color(0.45, 0.25, 0.45, 1.0),
		"button_pressed": Color(0.25, 0.15, 0.35, 1.0),
	},
	
	ThemeType.CYBER: {
		"background": Color(0.05, 0.02, 0.1, 1.0),
		"surface": Color(0.1, 0.02, 0.15, 1.0),
		"primary": Color(0.0, 0.9, 0.8, 1.0),
		"secondary": Color(0.9, 0.0, 0.8, 1.0),
		"accent": Color(0.5, 0.0, 0.9, 1.0),
		"text": Color(0.9, 0.95, 1.0, 1.0),
		"text_dim": Color(0.5, 0.55, 0.6, 1.0),
		"slider_track": Color(0.15, 0.05, 0.2, 1.0),
		"slider_fill": Color(0.0, 0.9, 0.8, 1.0),
		"button_normal": Color(0.1, 0.05, 0.2, 1.0),
		"button_hover": Color(0.15, 0.08, 0.25, 1.0),
		"button_pressed": Color(0.08, 0.02, 0.15, 1.0),
	}
}

func _ready() -> void:
	# Apply default theme
	apply_theme(current_theme)

func switch_theme(theme_type: ThemeType) -> void:
	current_theme = theme_type
	apply_theme(theme_type)

func apply_theme(theme_type: ThemeType) -> void:
	var theme_colors = themes[theme_type]
	var main_control = get_parent()
	
	if main_control and main_control.has_node("Background"):
		var bg = main_control.get_node("Background")
		bg.color = theme_colors["background"]
	
	# Update all labels
	if main_control:
		_update_label_colors(main_control, theme_colors)
	
	print("Applied theme: ", ThemeType.keys()[theme_type])

func _update_label_colors(node: Node, colors: Dictionary) -> void:
	if node is Label:
		var label = node as Label
		label.add_theme_color_override("font_color", colors["text"])
		label.add_theme_color_override("font_color_uneditable", colors["text_dim"])
	
	for child in node.get_children():
		_update_label_colors(child, colors)
