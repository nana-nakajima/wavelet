# WAVELET Tonverk Parity Implementation Plan

## Overview

This document outlines the implementation roadmap to achieve full Tonverk feature parity for WAVELET. The plan is structured in phases, with each phase building upon the previous one.

## Phase 0: Foundation (Architecture & Infrastructure)

### Goals
- Redesign web UI to Tonverk-style track-based architecture
- Establish proper Rust DSP architecture matching Tonverk signal flow

### Tasks

#### 0.1 Web UI Architecture Redesign
- [ ] Create 16-track container component
- [ ] Implement track selector/column UI
- [ ] Create page navigation system (TRIG → SRC → FLTR → AMP → FX → MOD)
- [ ] Build 8-knob rotary encoder component with OLED-style display
- [ ] Implement step sequencer grid (16 buttons with LED states)
- [ ] Design transport bar (Play/Stop/Record, BPM, pattern, song position)
- [ ] Add mute/solo buttons per track

#### 0.2 Rust Architecture Redesign
- [ ] Restructure synth to 16-track architecture
- [ ] Implement track types: Audio, Bus, Send FX, Mix
- [ ] Define proper routing system (MIX AB, OUT CD, OUT EF, BUS 1-4)
- [ ] Create machine abstraction layer for SRC/FX

### Dependencies
- None (foundation phase)

---

## Phase 1: Core Audio Engine (SRC → FLTR → AMP)

### Goals
- Implement all machine types (Single Player, Multi Player, Subtracks, MIDI)
- Implement dual filter system (Multimode + Base-width)
- Implement amp envelope with velocity

### Tasks

#### 1.1 Machine Types (SRC)

**Single Player:**
- [ ] Sample playback engine (mono/stereo)
- [ ] 8-voice polyphony
- [ ] Parameters: TUNE (±5 oct), PLAY MODE (Fwd/Rev/FwdLoop/RevLoop)
- [ ] LOOP CROSSFADE, SAMPLE SLOT, STRT, END, LSTR, LEND
- [ ] Sample slot management (1023 slots per project)

**Multi Player:**
- [ ] Multi-sampled instrument support
- [ ] 8-voice polyphony
- [ ] Parameters: TUNE, VIBR, SPD, FADE

**Subtracks:**
- [ ] 8 independent subtracks with separate SRC/FLTR/AMP/MOD
- [ ] Supertrack for shared FX
- [ ] Independent parameter sets per subtrack

**MIDI Machine:**
- [ ] External MIDI control (tracks 1-12)
- [ ] CHAN, BANK, SBNK, PROG, PB, AT, MW, BC
- [ ] 16 assignable CC commands

#### 1.2 Filter System

**Multimode Filter (FLTR Page 1):**
- [ ] Morph LP → BP → HP
- [ ] Parameters: ATK, DEC, SUS, REL, FREQ, RESO, TYPE, ENV

**Base-width Filter (FLTR Page 2):**
- [ ] HP + LP in series
- [ ] Parameters: BASE, WIDTH, DEL, SPRD, KEY.T, RSET
- [ ] Filter reordering support

#### 1.3 Amplifier
- [ ] ADSR envelope per voice
- [ ] Velocity to volume
- [ ] Overdrive/clipping
- [ ] Stereo panning

### Dependencies
- Phase 0

---

## Phase 2: Effects System

### Goals
- Implement all Tonverk insert effects (11 total)
- Implement bus/send/mix-specific effects
- Create proper FX slot architecture (2 slots per insert track)

### Tasks

#### 2.1 Insert FX (Tracks 1-8, 2 slots each)

**Chrono Pitch (Granular Pitch Shifter):**
- [ ] TUNE, WIN, FDBK, DEP, HPF, LPF, SPD, MIX

**Comb ± Filter:**
- [ ] SPD, DEP, SPH, DTUN, FREQ, FDBK, LPF, MIX

**Compressor:**
- [ ] THR, ATK, REL, MUP, RAT (1.50-20.00)
- [ ] SCS (sidechain source), SCF (sidechain filter), MIX

**Degrader (Lo-fi):**
- [ ] BR (16-1 bit), OVER, SRR, DROP, RATE
- [ ] DEP, FREZ, F.TIM

**Dirtshaper (Distortion):**
- [ ] DRV, RECT, HPF, LPF, NOIS, N.FRQ, N.RES, MIX

**Filterbank (8-band Fixed):**
- [ ] Gain A-H (90Hz → 4kHz HP)

**Infinite Flanger (Barber-pole):**
- [ ] SPD, DEP, TUNE, FDBK, LPF

**Low-pass Filter (4-pole 24dB/oct):**
- [ ] SPD, DEP, SPH, LAG, FREQ, RESO, SPRD

**Panoramic Chorus:**
- [ ] DEP, SPD, HPF, WDTH, MIX

**Phase 98 (4/6-stage Phaser):**
- [ ] SPD, DEP, SHP, LAG, FREQ, FDBK, STG, MIX

**Saturator Delay:**
- [ ] TIME (1/128 to 1 bar), X, WID, FDBK, HPF, LPF, MIX

#### 2.2 Bus FX (Tracks 9-12, 2 slots each)

All Insert FX plus:
- [ ] **Frequency Warper:** SPD, DEP, SPH, LAG, SHFT, SPRD, SBND, MIX
- [ ] **Supervoid Reverb:** PRE, DEC, FREQ, GAIN, HPF, LPF, MIX
- [ ] **Warble:** SPEED, DEPTH, BASE, WIDTH, N.LEV, N.HPF, STEREO, MIX

#### 2.3 Send FX (Tracks 13-15, 1 slot each)

- [ ] Compressor
- [ ] Daisy Delay: DRV, TIME, FDBK, WIDH, MOD, SKEW, FILT
- [ ] Panoramic Chorus
- [ ] Rumsklang Reverb: PRE, EARLY, DAMP, SIZE, LOWC, HIGHC
- [ ] Saturator Delay
- [ ] Supervoid Reverb

#### 2.4 Mix FX (Track 16, 1 slot)

All Bus FX plus:
- [ ] Multimode Filter

### Dependencies
- Phase 0, Phase 1.1-1.2

---

## Phase 3: Modulation System

### Goals
- Implement per-voice LFOs (Voice LFO 1 & 2)
- Implement FX LFOs (FX LFO 1 & 2)
- Implement Modulation Envelope
- Create modulation routing matrix

### Tasks

#### 3.1 LFO Implementation

**Voice LFOs (per audio track):**
- [ ] Per-voice instantiation (8 copies for 8 voices)
- [ ] SPD (tempo-synced steps), MULT (1-2K)
- [ ] FADE (bipolar -64 to +63)
- [ ] DEST (extensive mod destinations)
- [ ] WAVE (Tri/Sine/Sq/Saw/Random/Exp/Ramp)
- [ ] SPH (start phase), MODE (FREE/TRIG/HOLD/ONE/HALF)
- [ ] DEP (bipolar depth)

**FX LFOs:**
- [ ] Independent of voice count
- [ ] Same parameters as Voice LFOs
- [ ] Modulate FX parameters

#### 3.2 Mod Envelope
- [ ] ADSR envelope for modulation routing
- [ ] Parameters: ATK, DEC, SUS, REL, RSET, DEST, DEP

#### 3.3 Modulation Matrix
- [ ] Connect LFO/Env sources to destinations
- **Destinations:**
  - SRC parameters (8 knobs)
  - FLTR: Type, Freq, Reso, Spread, EnvDepth, EnvDelay, ATK, DEC, SUS, REL, Base, Width
  - AMP: ATK, Hold, DEC, SUS, REL, Overdrive, Pan, Volume
  - FX: 16 knobs across 2 FX slots
  - Routing: Output, Send 1-3 Amount

### Dependencies
- Phase 0, Phase 1, Phase 2

---

## Phase 4: Sequencer

### Goals
- Implement full 256-step sequencer
- Implement parameter locks per trig
- Implement trig conditions and micro timing

### Tasks

#### 4.1 Core Sequencer
- [ ] 16 pages × 16 steps = 256 steps max
- [ ] Per-page LENGTH (1-16 steps)
- [ ] Per-page SCALE (1/8 to 2×)
- [ ] Pattern change and reset timing

#### 4.2 Trig Types
- [ ] Note Trig (red LED)
- [ ] Lock Trig (yellow LED)
- [ ] Combined Trig (blinking red LED)

#### 4.3 Parameter Locks
- [ ] Lock any parameter to per-step value
- [ ] Visual indication of locked parameters
- [ ] Real-time parameter lock recording

#### 4.4 Trig Conditions
- [ ] FILL/FILL̄ (fill mode active/inactive)
- [ ] PRE/PRĒ (previous trig)
- [ ] NEI/NEĪ (neighbor track)
- [ ] 1ST/1ST̄ (first loop only)
- [ ] LST/LST̄ (last pattern play)
- [ ] A:B/A:B̄ (every A out of B loops)

#### 4.5 Micro Timing
- [ ] Per-step timing offset
- [ ] Large increments (LEFT/RIGHT)
- [ ] Small increments (UP/DOWN)

#### 4.6 Retrigs
- [ ] RTRG (enable/disable)
- [ ] RTIM (1/128 to 1/1, triplets)
- [ ] RVEL (velocity curve fade)

### Dependencies
- Phase 0, Phase 1, Phase 2, Phase 3

---

## Phase 5: Arpeggiator

### Goals
- Implement full arpeggiator per track

### Tasks
- [ ] MODE (pattern mode)
- [ ] SPEED
- [ ] RANGE (octaves)
- [ ] N.LEN (note length)
- [ ] OFFSET
- [ ] ARP LENGTH

### Dependencies
- Phase 0

---

## Phase 6: Song Mode

### Goals
- Implement song arrangement system

### Tasks
- [ ] Up to 16 songs per project
- [ ] Up to 99 rows per song
- **Per-row parameters:**
  - LABEL (Verse/Chorus/Bridge/etc.)
  - PTN (pattern selection)
  - ROW PLAY COUNT
  - ROW LENGTH (2-1024 steps)
  - ROW TEMPO (BPM per row)
  - END (LOOP/STOP)

### Dependencies
- Phase 4

---

## Phase 7: Perform Mode

### Goals
- Implement temporary parameter tweaks

### Tasks
- [ ] Temporary memory for parameter changes
- [ ] Changes not saved to pattern
- [ ] Revert on exit
- [ ] Active on one pattern at a time

### Dependencies
- Phase 1, Phase 2, Phase 3

---

## Phase 8: Sampling System

### Goals
- Implement audio recording
- Implement Auto Sampler for multi-sampled instruments

### Tasks

#### 8.1 Recorder
- [ ] Maximum recording: 6:06:06
- [ ] REC (start/stop)
- [ ] ARM (threshold-triggered)
- [ ] RLEN (1/16 to MAX)
- [ ] THR (threshold level)
- [ ] SRC (IN A+USB L, IN B+USB R, IN+USB, MAIN, TRK 1-8, BUS 1-4)
- [ ] MON (monitor)

#### 8.2 Auto Sampler
- [ ] START/END note range
- [ ] SAMPLE EVERY (interval)
- [ ] VELOCITY LAYERS
- [ ] NOTE DURATION
- [ ] LTNCY (latency compensation)
- [ ] RELEASE TIME

#### 8.3 Sample Management
- [ ] WAV/AIFF formats
- [ ] 16/24/32-bit depth
- [ ] 48kHz native (44.1/88.2/96kHz support)
- [ ] Up to 4GB project RAM
- [ ] 1023 sample slots per project

### Dependencies
- Phase 0, Phase 1.1

---

## Phase 9: MIDI System

### Goals
- Implement MIDI configuration
- Implement MIDI machine

### Tasks

#### 9.1 MIDI Configuration
- [ ] Sync: Clock/transport receive/send
- [ ] Program change receive/send
- **Port Config:**
  - Receive notes
  - Receive CC
  - Input: MIDI/USB/MIDI+USB
  - Output configuration
- **Channels:**
  - AUTO CHANNEL
  - Per-track MIDI channels (1-16)

#### 9.2 MIDI Machine (Tracks 1-12)
- [ ] CHAN (MIDI channel 1-16)
- [ ] BANK, SBNK (bank change)
- [ ] PROG (program change)
- [ ] PB (pitch bend)
- [ ] AT (aftertouch)
- [ ] MW (mod wheel)
- [ ] BC (breath controller)
- [ ] 16 assignable CC commands

### Dependencies
- Phase 1.1 (MIDI Machine)

---

## Phase 10: Polish & Optimization

### Goals
- Audio performance optimization
- UI/UX refinements
- Testing

### Tasks
- [ ] Run `cargo test --lib` after all Rust changes
- [ ] Run `cargo bench` to verify performance
- [ ] Latency optimization
- [ ] CPU usage optimization
- [ ] Memory usage optimization
- [ ] Web UI responsiveness improvements
- [ ] Accessibility improvements
- [ ] Mobile layout support
- [ ] Keyboard shortcuts matching Tonverk

---

## Implementation Order Summary

| Phase | Focus | Duration Estimate |
|-------|-------|-------------------|
| 0 | Foundation (UI + Rust architecture) | 2-3 weeks |
| 1 | Core Audio (SRC, Filters, Amp) | 2-3 weeks |
| 2 | Effects System (15+ effects) | 3-4 weeks |
| 3 | Modulation System | 2 weeks |
| 4 | Sequencer (256 steps, param locks) | 2-3 weeks |
| 5 | Arpeggiator | 1 week |
| 6 | Song Mode | 1 week |
| 7 | Perform Mode | 1 week |
| 8 | Sampling System | 2 weeks |
| 9 | MIDI System | 1-2 weeks |
| 10 | Polish & Optimization | Ongoing |

**Total Estimated Time:** 17-22 weeks

---

## Key Dependencies Graph

```
Phase 0 (Foundation)
    ↓
Phase 1 (Core Audio) → Phase 5 (Arpeggiator)
    ↓
Phase 2 (Effects)
    ↓
Phase 3 (Modulation)
    ↓
Phase 4 (Sequencer) → Phase 6 (Song Mode)
    ↓
Phase 7 (Perform Mode)
    ↑
    ↑
Phase 8 (Sampling) ← Phase 9 (MIDI)
    ↓
Phase 10 (Polish)
```

---

## Success Criteria

Each phase should:
1. Pass all `cargo test --lib` tests
2. Pass `cargo bench` benchmarks
3. Have corresponding web UI implementation
4. Document parameter mappings from Tonverk manual
5. Include unit tests for new DSP components
