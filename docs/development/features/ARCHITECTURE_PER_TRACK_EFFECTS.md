# Architecture: Per-track Effects

**Version**: v1.0
**Date**: 2026-02-03
**Related**: PRODUCT_BRIEF_PER_TRACK_EFFECTS.md

---

## 1. Overview

The Per-track Effects module provides independent effect chain support for each track of the WAVELET sequencer.

### 1.1 Design Goals

1. **Lightweight**: Each track's effects use < 5% CPU
2. **Extensible**: Easy to add new effect types
3. **High Performance**: Real-time audio processing, lock-free design
4. **Compatible**: Seamless integration with existing Effect trait system

### 1.2 Architecture Principles

```
┌─────────────────────────────────────────────────────────────┐
│                    Track Effects System                      │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐   │
│  │ Track 0 │    │ Track 1 │    │ Track 2 │    │ Track 3 │   │
│  ├─────────┤    ├─────────┤    ├─────────┤    ├─────────┤   │
│  │ Filter  │    │ Distort │    │   EQ    │    │  Comp   │   │
│  │ Distort │    │   EQ    │    │  Comp   │    │  Delay  │   │
│  │   ...   │    │   ...   │    │   ...   │    │   ...   │   │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. System Architecture

### 2.1 Core Components

```
src/
├── effects/
│   ├── mod.rs           # Effect trait definition
│   ├── track_effects.rs # Per-track effects core (NEW)
│   ├── filter.rs        # Filter effect
│   ├── distortion.rs    # Distortion effect
│   ├── compressor.rs    # Compressor effect
│   └── simple_eq.rs     # Equalizer effect
├── step_sequencer.rs    # Sequencer (integrates TrackEffects)
└── lib.rs               # Module exports
```

### 2.2 Data Structures

#### TrackEffectSlot (Effect Slot)

```rust
/// Single effect slot
pub struct TrackEffectSlot {
    /// Effect type
    effect_type: Option<EffectType>,

    /// Effect instance
    effect: Option<Box<dyn Effect>>,

    /// Whether enabled
    enabled: bool,

    /// Mix ratio (0.0 = dry, 1.0 = wet)
    mix: f32,

    /// Parameter lock mapping
    param_locks: HashMap<ParameterId, f32>,
}
```

#### TrackEffects (Effect Chain)

```rust
/// Single track's effect chain
pub struct TrackEffects {
    /// Effect slots (up to 4)
    slots: [Option<TrackEffectSlot>; 4],

    /// Track ID
    track_id: u8,

    /// Enabled state
    enabled: bool,

    /// Bypass state
    bypass: bool,
}
```

#### PerTrackEffectsManager (Manager)

```rust
/// Global effects manager
pub struct PerTrackEffectsManager {
    /// All track effects (8 tracks)
    track_effects: [TrackEffects; 8],

    /// Shared effect factory
    effect_factory: EffectFactory,

    /// Cached presets
    presets: Vec<EffectChainPreset>,
}
```

---

## 3. Effect Factory Pattern

### 3.1 EffectFactory

```rust
/// Effect factory - creates and manages effect instances
pub struct EffectFactory {
    /// Sample rate
    sample_rate: f32,

    /// Effect type registry
    registered_types: HashMap<EffectType, EffectBuilder>,
}

impl EffectFactory {
    /// Create an effect of the specified type
    pub fn create_effect(&self, effect_type: EffectType) -> Option<Box<dyn Effect>> {
        self.registered_types
            .get(&effect_type)
            .and_then(|builder| builder(self.sample_rate))
    }

    /// Register a new effect type
    pub fn register<E: Effect + Default + 'static>(
        &mut self,
        effect_type: EffectType,
    ) {
        self.registered_types.insert(
            effect_type,
            Box::new(|sr| Box::new(E::new(sr))),
        );
    }
}
```

### 3.2 Supported Effect Types

| EffectType | Implementation | CPU Estimate | Purpose |
|------------|---------------|-------------|---------|
| Filter | BiquadFilter | ~1% | Tone shaping |
| Saturation | Saturation | ~1% | Distortion/warmth |
| Compressor | Compressor | ~2% | Dynamic control |
| SimpleEQ | SimpleEq | ~1% | Frequency adjustment |
| Chorus | Chorus | ~2% | Stereo width |
| Delay | Delay | ~2% | Spatial depth |

---

## 4. Data Flow

### 4.1 Audio Processing Flow

```
Input Sample
    │
    ▼
┌───────────────────┐
│  TrackEffects     │
│  ┌─────────────┐  │
│  │ Slot 0      │  │ ← if enabled
│  │ (Filter)    │──┤
│  └─────────────┘  │
│  ┌─────────────┐  │
│  │ Slot 1      │  │ ← if enabled
│  │ (Saturation)│──┤
│  └─────────────┘  │
│  ┌─────────────┐  │
│  │ Slot 2      │  │ ← if enabled
│  │ (Compressor)│──┤
│  └─────────────┘  │
└───────────────────┘
    │
    ▼
Output Sample
```

### 4.2 Real-time Parameter Updates

```
Parameter Change Event
    │
    ├───> Effect Slot (direct update)
    │
    └───> ParamLock Manager (record mapping)
             │
             └───> Apply on Next Step (sequencer sync)
```

---

## 5. Sequencer Integration

### 5.1 Modifying StepSequencer

```rust
pub struct StepSequencer {
    // ... existing fields ...

    /// Track effects
    track_effects: PerTrackEffectsManager,
}

impl StepSequencer {
    /// Process a single track's audio output
    fn process_track_output(&mut self, track_id: u8, sample: f32) -> f32 {
        // 1. Normal sequencer processing
        let mut sample = self.tracks[track_id as usize].process(sample);

        // 2. Apply effect chain
        sample = self.track_effects.process_track(track_id, sample);

        sample
    }
}
```

### 5.2 ParameterLock Integration

```rust
impl TrackEffects {
    /// Apply parameter locks to effects
    pub fn apply_param_locks(&mut self, step: u8) {
        for slot in &mut self.slots {
            if let Some(ref mut effect) = slot.effect {
                // Find effect parameter locks for this step
                for (param_id, value) in &slot.param_locks {
                    self.apply_param_to_effect(effect.as_mut(), param_id, *value);
                }
            }
        }
    }
}
```

---

## 6. Performance Optimization Strategies

### 6.1 CPU Optimization

1. **Lazy loading of effects**: Only allocate memory when an effect is enabled
2. **SIMD optimization**: Use SIMD for compute-intensive effects like filters
3. **Cache-friendly**: Effect state uses stack allocation, avoiding frequent memory access
4. **Bypass optimization**: Skip all processing when an effect is disabled

### 6.2 Memory Optimization

```rust
// Use Option to avoid unnecessary allocation
struct TrackEffects {
    slots: [Option<TrackEffectSlot>; 4], // Only enabled effects will be Some
}

// Effect instances use Box, but share the same type
effect: Option<Box<dyn Effect>>,
```

### 6.3 Latency Optimization

- All effect processing completes within the same audio buffer
- No additional buffer copying required
- Estimated latency: < 0.5ms (44.1kHz @ 22 samples)

---

## 7. Error Handling

### 7.1 Effect Creation Failure

```rust
impl TrackEffects {
    pub fn add_effect(&mut self, slot_index: usize, effect_type: EffectType) -> Result<(), Error> {
        if slot_index >= 4 {
            return Err(Error::InvalidSlotIndex);
        }
        
        let effect = self.factory.create_effect(effect_type)
            .ok_or(Error::UnsupportedEffect)?;
            
        self.slots[slot_index] = Some(TrackEffectSlot {
            effect_type: Some(effect_type),
            effect: Some(effect),
            enabled: true,
            mix: 0.5,
            param_locks: HashMap::new(),
        });
        
        Ok(())
    }
}
```

### 7.2 State Recovery

```rust
impl TrackEffects {
    /// Restore effect state from snapshot
    pub fn from_snapshot(snapshot: &TrackEffectsSnapshot) -> Self {
        let mut effects = Self::new(snapshot.track_id);
        
        for (i, slot_snapshot) in snapshot.slots.iter().enumerate() {
            if let Some(ref effect_type) = slot_snapshot.effect_type {
                effects.add_effect(i, *effect_type).unwrap();
                
                if let Some(ref mut slot) = effects.slots[i] {
                    slot.enabled = slot_snapshot.enabled;
                    slot.mix = slot_snapshot.mix;
                }
            }
        }
        
        effects
    }
}
```

---

## 8. Testing Strategy

### 8.1 Unit Tests

| Test Item | Coverage |
|-----------|----------|
| `test_track_effects_creation` | Creation and initialization |
| `test_add_remove_effects` | Adding/removing effects |
| `test_effect_processing` | Effect processing correctness |
| `test_bypass_behavior` | Bypass behavior |
| `test_mix_parameter` | Mix parameter |
| `test_param_locks` | Parameter lock integration |

### 8.2 Integration Tests

- Sequencer + effects integration
- 8-track effects performance test
- Effect switching without artifacts test

### 8.3 Performance Tests

```rust
#[test]
fn test_track_effects_performance() {
    let mut effects = PerTrackEffectsManager::new(44100.0);
    
    // Add effects to all tracks
    for track_id in 0..8 {
        effects.add_effect(track_id, 0, EffectType::Filter).unwrap();
        effects.add_effect(track_id, 1, EffectType::Saturation).unwrap();
    }

    // Performance test: 1000 iterations
    let start = Instant::now();
    for _ in 0..1000 {
        for track_id in 0..8 {
            let _ = effects.process_track(track_id, 0.5);
        }
    }
    let duration = start.elapsed();

    // Should complete within a reasonable time
    assert!(duration.as_secs_f32() < 0.1);
}
```

---

## 9. Future Extensions

### 9.1 Extensible Effects

```rust
// Register new effects
effect_factory.register::<CustomEffect>(EffectType::Custom);

// Effects automatically appear in the UI selection list
```

### 9.2 Effect Preset System

```rust
pub struct EffectChainPreset {
    name: String,
    author: String,
    slots: [Option<EffectSlotConfig>; 4],
    tags: Vec<String>,
}

// Presets can be saved/shared
```

### 9.3 Advanced Effects

- Reverb (convolution or algorithmic reverb)
- Delay (stereo delay)
- Phaser (phase effect)
- Granular (granular effect)

---

## 10. Decision Log

| Decision | Options | Choice | Rationale |
|----------|---------|--------|-----------|
| Number of effect slots | 4 / 8 | 4 | Balance between features and CPU |
| Effect execution order | Fixed/Adjustable | Fixed (Filter->Dist->EQ->Comp) | Simplify UI, reduce complexity |
| Parameter lock granularity | Step/Bar | Step | Consistent with existing sequencer system |
| Effect types | Basic/Advanced | Basic (P0 priority) | Deliver core functionality quickly |

---

*This document was created based on the WAVELET development workflow (BMAD-METHOD)*
