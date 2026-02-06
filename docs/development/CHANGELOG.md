# Changelog

## [2.1.0] - 2026-02-02

### Added
- **Zero-Delay Feedback (ZDF) Filter** (`src/filter.rs`)
  - New `ZdfFilter` struct implementing ladder filter topology
  - Zero-delay feedback structure for analog character
  - Supports LowPass2, LowPass4, and HighPass2 modes
  - Parameters: cutoff, resonance, drive
  - More analog-sounding than biquad filters

- **Analog Saturation Effect** (`src/effects/saturation.rs`)
  - New `Saturation` effect with soft clipping
  - Parameters: drive, tone, mix
  - Waveshaper curve similar to analog circuits
  - Adds harmonic content and warmth

- **Switchable Effects** (`src/effects/mod.rs`)
  - Added `enabled: bool` field to all effects
  - Added `process_with_bypass()` method for bypass support
  - Updated `EffectChain` to respect enabled state
  - `is_enabled()` and `set_enabled()` methods on all effects

- **Oversampling for Oscillators** (`src/oscillator.rs`)
  - Added `OversampleFactor` enum (1x, 2x, 4x, 8x)
  - Polyphase interpolation for upsampling/downsampling
  - Reduces aliasing in Saw/Square waveforms

- **Updated Synth Parameters** (`src/synth.rs`)
  - New parameter IDs for VA features:
    - `PARAM_ZDF_ENABLED` (50)
    - `PARAM_ZDF_CUTOFF` (51)
    - `PARAM_ZDF_RES` (52)
    - `PARAM_ZDF_DRIVE` (53)
    - `PARAM_SATURATION_DRIVE` (54)
    - `PARAM_SATURATION_MIX` (55)
    - `PARAM_OVERSAMPLE` (56)

### Changed
- Updated `lib.rs` to export new VA types
- Version bump to 2.1.0
- All effects now support bypass via `process_with_bypass()`

### Fixed
- Various minor improvements and bug fixes

## [2.0.0] - Initial Release
- Initial release of WAVELET synthesizer
- Core oscillator, filter, envelope, LFO modules
- Effects: Reverb, Delay, Distortion, Compressor
- Polyphonic synthesizer engine
