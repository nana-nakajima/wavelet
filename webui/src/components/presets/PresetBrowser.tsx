import React, { useState } from 'react';
import { useTonverkStore } from '../tonverkStore';
import { useAudio } from '../context/AudioContext';

interface Preset {
  id: string;
  name: string;
  category: 'lead' | 'pad' | 'bass' | 'keys' | 'drums' | 'fx' | 'arp';
  author: string;
  tags: string[];
  difficulty: 'beginner' | 'intermediate' | 'pro';
  description: string;
}

const PRESETS: Preset[] = [
  // Leads (10)
  { id: 'lead-1', name: 'SUPER SAW', category: 'lead', author: 'WAVELET', tags: ['synth', 'brass', 'warm'], difficulty: 'beginner', description: 'Classic supersaw lead' },
  { id: 'lead-2', name: 'ACID BURN', category: 'lead', author: 'WAVELET', tags: ['303', 'acid', 'squelchy'], difficulty: 'intermediate', description: 'Resonant saw with filter drive' },
  { id: 'lead-3', name: 'DIGITAL PUNCH', category: 'lead', author: 'WAVELET', tags: ['digital', 'pluck', 'aggressive'], difficulty: 'beginner', description: 'Punchy digital lead' },
  { id: 'lead-4', name: 'VAPOR BELL', category: 'lead', author: 'WAVELET', tags: ['bell', 'digital', 'glassy'], difficulty: 'beginner', description: 'Bright bell-like lead' },
  { id: 'lead-5', name: 'LACEY SAW', category: 'lead', author: 'WAVELET', tags: ['saw', 'synth', 'pads'], difficulty: 'intermediate', description: 'Detuned saw stack' },
  { id: 'lead-6', name: 'PWM METAL', category: 'lead', author: 'WAVELET', tags: ['pwm', 'metallic', 'aggressive'], difficulty: 'pro', description: 'PWM with metallic character' },
  { id: 'lead-7', name: 'SINE SWEEP', category: 'lead', author: 'WAVELET', tags: ['sine', 'sweep', 'clean'], difficulty: 'beginner', description: 'Simple sine sweep' },
  { id: 'lead-8', name: 'WOBBLE KING', category: 'lead', author: 'WAVELET', tags: ['wobble', 'dubstep', 'filter'], difficulty: 'intermediate', description: 'LFO-modulated filter wobble' },
  { id: 'lead-9', name: 'TRANCE PLUCK', category: 'lead', author: 'WAVELET', tags: ['pluck', 'trance', 'gate'], difficulty: 'beginner', description: 'Gated trance pluck' },
  { id: 'lead-10', name: 'NEON PULSE', category: 'lead', author: 'WAVELET', tags: ['pulse', 'retro', '80s'], difficulty: 'intermediate', description: 'Retro pulse wave' },

  // Pads (8)
  { id: 'pad-1', name: 'WARM BED', category: 'pad', author: 'WAVELET', tags: ['warm', 'strings', 'background'], difficulty: 'beginner', description: 'Warm string pad' },
  { id: 'pad-2', name: 'SPACE DRONE', category: 'pad', author: 'WAVELET', tags: ['drone', 'space', 'ambient'], difficulty: 'beginner', description: 'Deep space drone' },
  { id: 'pad-3', name: 'ETHereal', category: 'pad', author: 'WAVELET', tags: ['ethereal', 'angelic', 'chorus'], difficulty: 'intermediate', description: 'Wide chorus pad' },
  { id: 'pad-4', name: 'DARK MATTER', category: 'pad', author: 'WAVELET', tags: ['dark', 'deep', 'cinematic'], difficulty: 'intermediate', description: 'Dark cinematic pad' },
  { id: 'pad-5', name: 'SHIMMER HEAVEN', category: 'pad', author: 'WAVELET', tags: ['shimmer', 'reverb', 'ambient'], difficulty: 'intermediate', description: 'Reverb-washed pad' },
  { id: 'pad-6', name: 'BOWED GLASS', category: 'pad', author: 'WAVELET', tags: ['glass', 'bowed', 'texture'], difficulty: 'pro', description: 'Bowed glass texture' },
  { id: 'pad-7', name: 'CHORAL SPLIT', category: 'pad', author: 'WAVELET', tags: ['chorus', 'split', 'vocals'], difficulty: 'intermediate', description: 'Split chorus effect' },
  { id: 'pad-8', name: 'OVERTONE', category: 'pad', author: 'WAVELET', tags: ['overtone', 'harmonic', 'texture'], difficulty: 'pro', description: 'Harmonic overtones' },

  // Basses (8)
  { id: 'bass-1', name: 'FAT SUB', category: 'bass', author: 'WAVELET', tags: ['sub', 'warm', 'fat'], difficulty: 'beginner', description: 'Fat sub bass' },
  { id: 'bass-2', name: 'GRITTY GRINDER', category: 'bass', author: 'WAVELET', tags: ['grit', 'distortion', 'house'], difficulty: 'intermediate', description: 'Distorted bass' },
  { id: 'bass-3', name: 'RESO PUNCH', category: 'bass', author: 'WAVELET', tags: ['resonant', 'punch', 'techno'], difficulty: 'intermediate', description: 'Resonant punch bass' },
  { id: 'bass-4', name: 'FM DEEP', category: 'bass', author: 'WAVELET', tags: ['fm', 'deep', 'digital'], difficulty: 'pro', description: 'FM deep bass' },
  { id: 'bass-5', name: 'WOBBLE BASS', category: 'bass', author: 'WAVELET', tags: ['wobble', 'dubstep', 'filter'], difficulty: 'intermediate', description: 'LFO wobble bass' },
  { id: 'bass-6', name: 'RETRO 808', category: 'bass', author: 'WAVELET', tags: ['808', 'retro', 'classic'], difficulty: 'beginner', description: 'Classic 808 bass' },
  { id: 'bass-7', name: 'GLITCH BASS', category: 'bass', author: 'WAVELET', tags: ['glitch', 'digital', 'textured'], difficulty: 'pro', description: 'Glitchy bass' },
  { id: 'bass-8', name: 'SINE SWELL', category: 'bass', author: 'WAVELET', tags: ['sine', 'swell', 'clean'], difficulty: 'beginner', description: 'Clean sine bass' },

  // Keys (6)
  { id: 'keys-1', name: 'EPIANO WARM', category: 'keys', author: 'WAVELET', tags: ['electric', 'piano', 'warm'], difficulty: 'beginner', description: 'Warm electric piano' },
  { id: 'keys-2', name: 'DX BELL', category: 'keys', author: 'WAVELET', tags: ['dx', 'bell', 'fm'], difficulty: 'intermediate', description: 'DX-style bell' },
  { id: 'keys-3', name: 'ORGAN DRAW', category: 'keys', author: 'WAVELET', tags: ['organ', 'drawbar', 'church'], difficulty: 'intermediate', description: 'Drawbar organ' },
  { id: 'keys-4', name: 'SYNTH CLAV', category: 'keys', author: 'WAVELET', tags: ['clav', 'synth', 'tine'], difficulty: 'beginner', description: 'Synthesized clavinet' },
  { id: 'keys-5', name: 'PIANO STRINGS', category: 'keys', author: 'WAVELET', tags: ['piano', 'strings', 'layer'], difficulty: 'intermediate', description: 'Piano with strings' },
  { id: 'keys-6', name: 'ELEC GRAND', category: 'keys', author: 'WAVELET', tags: ['electric', 'grand', 'realistic'], difficulty: 'pro', description: 'Electric grand' },

  // Drums (8)
  { id: 'drums-1', name: 'KICK PUNCH', category: 'drums', author: 'WAVELET', tags: ['kick', 'punch', 'house'], difficulty: 'beginner', description: 'Punchy kick' },
  { id: 'drums-2', name: 'SNAP CRACK', category: 'drums', author: 'WAVELET', tags: ['snare', 'crack', 'pop'], difficulty: 'beginner', description: 'Snap snare' },
  { id: 'drums-3', name: 'HAT CRISP', category: 'drums', author: 'WAVELET', tags: ['hihat', 'crisp', 'open'], difficulty: 'beginner', description: 'Crisp closed hat' },
  { id: 'drums-4', name: 'CLAP RIM', category: 'drums', author: 'WAVELET', tags: ['clap', 'rim', 'trap'], difficulty: 'beginner', description: 'Clap with rim' },
  { id: 'drums-5', name: 'TOM DEEP', category: 'drums', author: 'WAVELET', tags: ['tom', 'deep', 'room'], difficulty: 'beginner', description: 'Deep tom' },
  { id: 'drums-6', name: 'PERC TEXTURE', category: 'drums', author: 'WAVELET', tags: ['perc', 'texture', 'shaker'], difficulty: 'intermediate', description: 'Textured percussion' },
  { id: 'drums-7', name: 'KICK SUB', category: 'drums', author: 'WAVELET', tags: ['kick', 'sub', 'deep'], difficulty: 'beginner', description: 'Sub kick' },
  { id: 'drums-8', name: 'CYMBAL WASH', category: 'drums', author: 'WAVELET', tags: ['cymbal', 'crash', 'wash'], difficulty: 'beginner', description: 'Crash cymbal' },

  // FX (6)
  { id: 'fx-1', name: 'SWEEP UP', category: 'fx', author: 'WAVELET', tags: ['sweep', 'up', 'transition'], difficulty: 'beginner', description: 'Filter sweep up' },
  { id: 'fx-2', name: 'DOWNLIFTER', category: 'fx', author: 'WAVELET', tags: ['down', 'lifter', 'transition'], difficulty: 'beginner', description: 'Downward sweep' },
  { id: 'fx-3', name: 'GLITCH STORM', category: 'fx', author: 'WAVELET', tags: ['glitch', 'storm', 'digital'], difficulty: 'intermediate', description: 'Glitch explosion' },
  { id: 'fx-4', name: 'PHASER SWOOP', category: 'fx', author: 'WAVELET', tags: ['phaser', 'swoop', 'filter'], difficulty: 'intermediate', description: 'Phaser sweep' },
  { id: 'fx-5', name: 'STUTTER BREAK', category: 'fx', author: 'WAVELET', tags: ['stutter', 'break', 'glitch'], difficulty: 'intermediate', description: 'Stutter break' },
  { id: 'fx-6', name: 'REVERSE REVEAL', category: 'fx', author: 'WAVELET', tags: ['reverse', 'reveal', 'transition'], difficulty: 'beginner', description: 'Reverse cymbal' },

  // Arpeggios (4)
  { id: 'arp-1', name: 'TRANCE STEPS', category: 'arp', author: 'WAVELET', tags: ['trance', 'steps', 'gate'], difficulty: 'beginner', description: 'Trance arp' },
  { id: 'arp-2', name: 'HOUSE OHM', category: 'arp', author: 'WAVELET', tags: ['house', 'ohm', '4-on-floor'], difficulty: 'beginner', description: 'House ohm-ah' },
  { id: 'arp-3', name: 'SYNTH SEQUENCE', category: 'arp', author: 'WAVELET', tags: ['sequence', 'synth', 'melodic'], difficulty: 'intermediate', description: 'Melodic sequence' },
  { id: 'arp-4', name: 'EIGHTIES ARPEGGIO', category: 'arp', author: 'WAVELET', tags: ['80s', 'arpeggio', 'retro'], difficulty: 'intermediate', description: '80s style arp' },
];

const CATEGORIES = [
  { id: 'all', name: 'ALL', color: '#ffffff' },
  { id: 'lead', name: 'LEAD', color: '#ff6b6b' },
  { id: 'pad', name: 'PAD', color: '#4ecdc4' },
  { id: 'bass', name: 'BASS', color: '#ffe66d' },
  { id: 'keys', name: 'KEYS', color: '#95e1d3' },
  { id: 'drums', name: 'DRUMS', color: '#f38181' },
  { id: 'fx', name: 'FX', color: '#aa96da' },
  { id: 'arp', name: 'ARP', color: '#fcbad3' },
];

export const PresetBrowser: React.FC = () => {
  const [selectedCategory, setSelectedCategory] = useState('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedPreset, setSelectedPreset] = useState<string | null>(null);

  const tracks = useTonverkStore(state => state.tracks);
  const selectedTrackId = useTonverkStore(state => state.selectedTrackId);
  const setSelectedTrack = useTonverkStore(state => state.setSelectedTrack);
  const showBrowser = useTonverkStore(state => state.showBrowser);
  const setShowBrowser = useTonverkStore(state => state.setShowBrowser);
  const updateTrackParam = useTonverkStore(state => state.updateTrackParam);

  const filteredPresets = PRESETS.filter(preset => {
    const matchesCategory = selectedCategory === 'all' || preset.category === selectedCategory;
    const matchesSearch = preset.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         preset.tags.some(tag => tag.toLowerCase().includes(searchQuery.toLowerCase()));
    return matchesCategory && matchesSearch;
  });

  const loadPreset = (preset: Preset) => {
    // Load preset to selected track
    setSelectedTrack(selectedTrackId || 1);

    // Apply preset parameters to track
    // This is simplified - in real implementation, you'd load full preset data
    console.log('Loading preset:', preset.name);

    // Example parameter settings (would be preset-specific in real implementation)
    if (preset.category === 'lead') {
      updateTrackParam(selectedTrackId || 1, 'src_tune', 64);
      updateTrackParam(selectedTrackId || 1, 'fltr_freq', 80);
      updateTrackParam(selectedTrackId || 1, 'fltr_reso', 30);
    } else if (preset.category === 'pad') {
      updateTrackParam(selectedTrackId || 1, 'src_tune', 64);
      updateTrackParam(selectedTrackId || 1, 'fltr_freq', 40);
      updateTrackParam(selectedTrackId || 1, 'fltr_reso', 10);
    } else if (preset.category === 'bass') {
      updateTrackParam(selectedTrackId || 1, 'src_tune', 64);
      updateTrackParam(selectedTrackId || 1, 'fltr_freq', 100);
      updateTrackParam(selectedTrackId || 1, 'fltr_reso', 50);
    }

    setSelectedPreset(preset.id);
  };

  if (!showBrowser) return null;

  return (
    <div className="preset-browser-overlay" onClick={() => setShowBrowser(false)}>
      <div className="preset-browser" onClick={(e) => e.stopPropagation()}>
        <div className="preset-browser-header">
          <h2>PRESET BROWSER</h2>
          <button className="preset-browser-close" onClick={() => setShowBrowser(false)}>×</button>
        </div>

        <div className="preset-browser-search">
          <input
            type="text"
            placeholder="Search presets..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="preset-search-input"
          />
        </div>

        <div className="preset-browser-categories">
          {CATEGORIES.map(cat => (
            <button
              key={cat.id}
              className={`preset-category-btn ${selectedCategory === cat.id ? 'active' : ''}`}
              style={{
                borderColor: selectedCategory === cat.id ? cat.color : 'transparent',
                color: selectedCategory === cat.id ? cat.color : '#888',
              }}
              onClick={() => setSelectedCategory(cat.id)}
            >
              {cat.name}
            </button>
          ))}
        </div>

        <div className="preset-browser-grid">
          {filteredPresets.map(preset => (
            <div
              key={preset.id}
              className={`preset-card ${selectedPreset === preset.id ? 'selected' : ''}`}
              onClick={() => loadPreset(preset)}
            >
              <div className="preset-card-name">{preset.name}</div>
              <div className="preset-card-category" style={{ color: CATEGORIES.find(c => c.id === preset.category)?.color }}>
                {preset.category.toUpperCase()}
              </div>
              <div className="preset-card-tags">
                {preset.tags.slice(0, 3).map(tag => (
                  <span key={tag} className="preset-tag">{tag}</span>
                ))}
              </div>
              <div className="preset-card-desc">{preset.description}</div>
            </div>
          ))}
        </div>

        <div className="preset-browser-footer">
          <span className="preset-count">{filteredPresets.length} presets</span>
          <span className="preset-info">Select a preset to load to track {selectedTrackId || 1}</span>
        </div>
      </div>
    </div>
  );
};
