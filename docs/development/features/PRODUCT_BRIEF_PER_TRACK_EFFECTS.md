# Product Brief: Per-track Effects

**Version**: v1.0
**Date**: 2026-02-03
**Status**: Phase 1 - Product Definition

---

## Overview

Add **Per-track Effects** functionality to WAVELET, allowing users to independently add effect chains to each track of the sequencer, giving each track unique sound processing capabilities.

## Problem Statement

Currently, all tracks in WAVELET's Step Sequencer share a single effect chain, making it impossible to set up independent effect processing for different instruments/timbres. This limits tonal richness and music production flexibility.

**Reference Devices**: Elektron Digitakt / OctatrACK - Each track has independent filters, effects, and parameter locks

## User Stories

1. **As a** music producer, **I want to** add compression to the drum track, **so that** I get a more powerful punch

2. **As a** music producer, **I want to** add distortion to the bass track, **so that** I increase the low-frequency impact

3. **As a** music producer, **I want to** add chorus to the melody track, **so that** the sound becomes wider and more spacious

4. **As a** music producer, **I want to** independently toggle effects per track, **so that** I can quickly compare the sound with and without effects

## Core Requirements

### P0 (Must Have)
1. Independent effect slots for each Track
2. Support for basic effect types:
   - **Filter** - Cutoff frequency + Resonance
   - **Distortion** (Distortion/Saturation)
   - **Compressor**
   - **Equalizer** (EQ - SimpleEQ)
3. Effect on/off control
4. Mix/Wet parameter control

### P1 (Should Have)
1. Effect parameter locks (integration with sequencer parameter lock system)
2. Effect preset saving
3. Effect bypass
4. Effect chain order adjustment

### P2 (Nice to Have)
1. Advanced effects (Reverb, Delay, Chorus)
2. Effect parallel/serial mode switching
3. Effect visualization (spectrum analysis)

## Success Criteria

- [ ] Each track can independently add/configure/toggle effects
- [ ] Effect processing adds no extra latency (< 1ms)
- [ ] CPU usage within acceptable range (single track effects < 5%)
- [ ] At least 20+ unit tests passing
- [ ] Seamless integration with existing sequencer system

## Constraints

### Technical Constraints
1. Rust audio engine, no runtime memory allocation
2. Compatible with existing Effect trait system
3. Maintain real-time audio performance (lock-free design)
4. Godot UI visualization support

### Time Constraints
- Development cycle: 1 week
- Milestones:
  - Day 1-2: Core architecture and data structures
  - Day 3-4: Effect integration
  - Day 5: UI integration and testing

## Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| CPU performance issues | High | Medium | Optimize effect algorithms, use SIMD |
| Complex integration with existing system | Medium | High | Follow existing Effect trait design |
| UI implementation complexity | Medium | Medium | Simplify UI design, implement in phases |

## Acceptance Criteria

### Functional Acceptance
- [ ] Sequencer can create tracks with effects
- [ ] Effects can be added/removed from tracks
- [ ] Effect parameters can be adjusted in real-time
- [ ] Effect on/off control works correctly
- [ ] Effects integrate with parameter lock system

### Performance Acceptance
- [ ] Effect processing latency < 1ms
- [ ] Single track effects CPU < 5%
- [ ] 8-track effects total CPU < 30%

### Test Acceptance
- [ ] Unit test coverage > 80%
- [ ] Integration tests cover core functionality
- [ ] No memory leaks

## Related Documents

- **Architecture**: `docs/ARCHITECTURE_PER_TRACK_EFFECTS.md`
- **PRD**: `docs/PRD_PER_TRACK_EFFECTS.md`
- **Stories**: `docs/STORIES_PER_TRACK_EFFECTS.md`

## References

- **Elektron Digitakt**: Independent per-track filters + effects
- **VCV Rack**: Modular effect chain system
- **Bitwig Studio**: Container-based effect chains

---

*This document was created based on the WAVELET development workflow (BMAD-METHOD)*
