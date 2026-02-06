# WAVEL WebUI

A web-based modular synthesizer interface for WAVEL, inspired by VCV Rack with a Dead Cells-inspired pixel art aesthetic.

## Features

- 🎮 **Modular Rack Layout** - Vertical module mounting like VCV Rack
- 🎨 **Pixel Art Design** - Industrial dark theme with high-contrast accents
- 🎛️ **Interactive Knobs** - Drag-to-adjust parameter controls
- 🔌 **Cable Connections** - Visual patch cable system
- 📦 **Module Types**
  - VCO (Oscillator) - Blue
  - VCF (Filter) - Green
  - ADSR Envelope - Orange
  - LFO - Purple
  - Delay, Reverb - Orange/Green
  - Compressor - Red
  - Output - Yellow

## Tech Stack

- **React 18** - UI framework
- **TypeScript** - Type safety
- **Vite** - Build tool
- **Zustand** - State management
- **Tailwind CSS** - Utility styling
- **Framer Motion** - Animations

## Getting Started

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build
```

## Architecture

```
src/
├── components/
│   ├── Header.tsx       # Top navigation bar
│   ├── Rack.tsx         # Main rack container
│   ├── ModuleBrowser.tsx # Left sidebar module picker
│   ├── ModulePanel.tsx  # Individual module UI
│   ├── Knob.tsx         # Rotary knob control
│   └── Port.tsx         # Input/output jacks
├── store.ts             # Global state management
├── App.tsx              # Root component
└── styles/              # Global styles
```

## Module System

Modules are defined in `MODULE_SPECS` with:
- `hp` - Horizontal pitch units (1HP = 25px)
- `color` - Accent color for the module
- `inputs/outputs` - Jack names
- `params` - Adjustable parameters

## Design System

- **Dark industrial palette** (#1a1a1a base)
- **Color-coded modules** by function
- **Pixel-perfect borders** with subtle shadows
- **Conic gradient knobs** with value indicators

## Future Enhancements

- [ ] Cable drawing between ports
- [ ] Audio engine integration
- [ ] Preset saving/loading
- [ ] MIDI controller support
- [ ] Mobile responsive design
- [ ] Theme customization

---

Built with ❤️ by Nana
