import { create } from 'zustand';

// Tonverk-aligned track types
export type TrackType = 'audio' | 'bus' | 'send' | 'mix';

// Page navigation
export type PageType = 'trig' | 'src' | 'fltr' | 'amp' | 'fx' | 'mod';

// Step trig types (matching Tonverk)
export type TrigType = 'none' | 'note' | 'lock' | 'combined';

// LFO waveforms (matching Tonverk)
export type LfoWaveform = 'tri' | 'sine' | 'sq' | 'saw' | 'random' | 'exp' | 'ramp';

// Track interface
export interface Track {
  id: number; // 1-16
  type: TrackType;
  name: string;
  muted: boolean;
  solo: boolean;
  volume: number; // 0-127
  pan: number; // -64 to +63
  currentPage: PageType;

  // SRC (Machine) parameters
  srcType: 'single' | 'multi' | 'subtrack' | 'midi';
  srcParams: Record<string, number>;

  // FLTR parameters
  fltrParams: Record<string, number>; // Multimode + Base-width

  // AMP parameters
  ampParams: Record<string, number>;

  // FX slots (2 for audio/bus, 1 for send/mix)
  fxSlots: FxSlot[];

  // MOD parameters (LFO 1, LFO 2, Mod Envelope)
  modParams: Record<string, number>;

  // Sequencer (256 steps max, 16 pages × 16 steps)
  sequencer: SequencerData;
}

export interface FxSlot {
  id: number; // 1-2
  type: string; // Effect type
  bypass: boolean;
  params: Record<string, number>;
}

export interface SequencerData {
  enabled: boolean; // Track-level trig enable
  length: number; // 1-16 (steps per page)
  scale: number; // 1/8 to 2x
  pages: SequencerPage[];
}

export interface SequencerPage {
  id: number; // 0-15
  steps: Step[];
}

export interface Step {
  id: number; // 0-15
  trigType: TrigType;
  note: number | null; // MIDI note (null for lock-only)
  velocity: number; // 0-127

  // Parameter locks (per step)
  paramLocks: Record<string, number>;

  // Micro timing (-127 to +127)
  microTiming: number;

  // Retrig
  retrig: boolean;
  retrigRate: number; // 1/128 to 1/1
  retrigVelCurve: number; // -1.00 to +1.00

  // Trig condition
  condition: TrigCondition | null;
}

export type TrigCondition =
  | 'none'
  | 'fill' | 'notFill'
  | 'pre' | 'notPre'
  | 'nei' | 'notNei'
  | 'first' | 'notFirst'
  | 'last' | 'notLast'
  | 'aB' | 'notAB';

// Transport state
export interface TransportState {
  playing: boolean;
  recording: boolean;
  tempo: number; // 20-300 BPM
  currentStep: number; // 0-255
  currentPage: number; // 0-15
  patternId: number; // 0-127
  patternBank: number; // A-H
  songPosition: number; // Song row position
  playMode: 'pattern' | 'song';
}

// Global state
interface TonverkState {
  // Tracks (1-16)
  tracks: Track[];

  // Transport
  transport: TransportState;

  // UI state
  selectedTrackId: number | null;
  selectedPatternId: number;
  viewMode: 'pattern' | 'song' | 'perform';
  showBrowser: boolean;

  // Actions
  setSelectedTrack: (id: number | null) => void;
  setTrackPage: (trackId: number, page: PageType) => void;
  updateTrackParam: (trackId: number, param: string, value: number) => void;
  toggleMute: (trackId: number) => void;
  toggleSolo: (trackId: number) => void;
  setStepValue: (trackId: number, pageId: number, stepId: number, field: string, value: number | null) => void;
  setStepTrigType: (trackId: number, pageId: number, stepId: number, trigType: TrigType) => void;
  setStepNote: (trackId: number, pageId: number, stepId: number, note: number | null) => void;
  addParamLock: (trackId: number, pageId: number, stepId: number, param: string, value: number) => void;
  removeParamLock: (trackId: number, pageId: number, stepId: number, param: string) => void;
  setTransport: (transport: Partial<TransportState>) => void;
  nextStep: () => void;
}

const createDefaultStep = (): Step => ({
  id: 0,
  trigType: 'none',
  note: null,
  velocity: 100,
  paramLocks: {},
  microTiming: 0,
  retrig: false,
  retrigRate: 1,
  retrigVelCurve: 0,
  condition: null,
});

const createDefaultSequencerPage = (): SequencerPage => ({
  id: 0,
  steps: Array.from({ length: 16 }, (_, i) => ({ ...createDefaultStep(), id: i })),
});

const createDefaultSequencer = (): SequencerData => ({
  enabled: false,
  length: 16,
  scale: 1,
  pages: Array.from({ length: 16 }, (_, i) => ({
    ...createDefaultSequencerPage(),
    id: i,
  })),
});

const createDefaultTrack = (id: number, type: TrackType, name: string): Track => ({
  id,
  type,
  name,
  muted: false,
  solo: false,
  volume: 100,
  pan: 0,
  currentPage: 'trig',
  srcType: 'single',
  srcParams: {
    tune: 64, // ±5 octaves centered at 64
    playMode: 0, // 0-3: Fwd, Rev, FwdLoop, RevLoop
    loopCrossfade: 0,
    sampleSlot: 0,
    strt: 0,
    end: 127,
    lstr: 0,
    lend: 127,
  },
  fltrParams: {
    // Multimode filter
    fltrType: 0, // 0-127 morph LP→BP→HP
    freq: 64,
    reso: 0,
    envDepth: 64,
    atk: 0,
    dec: 64,
    sus: 127,
    rel: 64,
    // Base-width filter
    base: 0,
    width: 127,
    bwDel: 0,
    bwSpread: 0,
    keyTrack: 64,
    bwRset: 0,
  },
  ampParams: {
    atk: 0,
    hold: 0,
    dec: 64,
    sus: 127,
    rel: 64,
    velToVol: 64,
    overdrive: 0,
    pan: 0,
  },
  fxSlots: [
    { id: 1, type: 'bypass', bypass: true, params: {} },
    { id: 2, type: 'bypass', bypass: true, params: {} },
  ],
  modParams: {
    // LFO 1
    lfo1Speed: 16,
    lfo1Mult: 4,
    lfo1Fade: 64,
    lfo1Wave: 0,
    lfo1Sph: 0,
    lfo1Mode: 0,
    lfo1Dep: 64,
    // LFO 2
    lfo2Speed: 16,
    lfo2Mult: 4,
    lfo2Fade: 64,
    lfo2Wave: 0,
    lfo2Sph: 0,
    lfo2Mode: 0,
    lfo2Dep: 64,
    // Mod Envelope
    modAtk: 0,
    modDec: 64,
    modSus: 127,
    modRel: 64,
    modRset: 0,
    modDep: 64,
  },
  sequencer: createDefaultSequencer(),
});

// Track name mapping
const TRACK_CONFIG: { type: TrackType; name: string }[] = [
  { type: 'audio', name: 'AUDIO 1' },
  { type: 'audio', name: 'AUDIO 2' },
  { type: 'audio', name: 'AUDIO 3' },
  { type: 'audio', name: 'AUDIO 4' },
  { type: 'audio', name: 'AUDIO 5' },
  { type: 'audio', name: 'AUDIO 6' },
  { type: 'audio', name: 'AUDIO 7' },
  { type: 'audio', name: 'AUDIO 8' },
  { type: 'bus', name: 'BUS 1' },
  { type: 'bus', name: 'BUS 2' },
  { type: 'bus', name: 'BUS 3' },
  { type: 'bus', name: 'BUS 4' },
  { type: 'send', name: 'SEND 1' },
  { type: 'send', name: 'SEND 2' },
  { type: 'send', name: 'SEND 3' },
  { type: 'mix', name: 'MIX' },
];

export const useTonverkStore = create<TonverkState>((set, get) => ({
  tracks: Array.from({ length: 16 }, (_, i) =>
    createDefaultTrack(i + 1, TRACK_CONFIG[i].type, TRACK_CONFIG[i].name)
  ),
  transport: {
    playing: false,
    recording: false,
    tempo: 120,
    currentStep: 0,
    currentPage: 0,
    patternId: 0,
    patternBank: 0,
    songPosition: 0,
    playMode: 'pattern',
  },
  selectedTrackId: null,
  selectedPatternId: 0,
  viewMode: 'pattern',
  showBrowser: false,

  setSelectedTrack: (id) => set({ selectedTrackId: id }),

  setTrackPage: (trackId, page) =>
    set((state) => ({
      tracks: state.tracks.map((t) =>
        t.id === trackId ? { ...t, currentPage: page } : t
      ),
    })),

  updateTrackParam: (trackId, param, value) =>
    set((state) => ({
      tracks: state.tracks.map((t) =>
        t.id === trackId
          ? {
              ...t,
              srcParams: param.startsWith('src') ? { ...t.srcParams, [param.substring(3)]: value } : t.srcParams,
              fltrParams: param.startsWith('fltr') ? { ...t.fltrParams, [param.substring(5)]: value } : t.fltrParams,
              ampParams: param.startsWith('amp') ? { ...t.ampParams, [param.substring(3)]: value } : t.ampParams,
              modParams: param.startsWith('mod') ? { ...t.modParams, [param.substring(3)]: value } : t.modParams,
            }
          : t
      ),
    })),

  toggleMute: (trackId) =>
    set((state) => ({
      tracks: state.tracks.map((t) =>
        t.id === trackId ? { ...t, muted: !t.muted } : t
      ),
    })),

  toggleSolo: (trackId) =>
    set((state) => ({
      tracks: state.tracks.map((t) =>
        t.id === trackId ? { ...t, solo: !t.solo } : t
      ),
    })),

  setStepValue: (trackId, pageId, stepId, field, value) =>
    set((state) => ({
      tracks: state.tracks.map((t) => {
        if (t.id !== trackId) return t;
        const page = t.sequencer.pages[pageId];
        const step = page.steps[stepId];
        return {
          ...t,
          sequencer: {
            ...t.sequencer,
            pages: t.sequencer.pages.map((p) =>
              p.id === pageId
                ? {
                    ...p,
                    steps: p.steps.map((s) =>
                      s.id === stepId
                        ? { ...s, [field]: value }
                        : s
                    ),
                  }
                : p
            ),
          },
        };
      }),
    })),

  setStepTrigType: (trackId, pageId, stepId, type) =>
    set((state) => ({
      tracks: state.tracks.map((t) => {
        if (t.id !== trackId) return t;
        return {
          ...t,
          sequencer: {
            ...t.sequencer,
            pages: t.sequencer.pages.map((p) =>
              p.id === pageId
                ? {
                    ...p,
                    steps: p.steps.map((s) =>
                      s.id === stepId
                        ? {
                            ...s,
                            trigType: type,
                            note: type === 'note' ? 60 : (type === 'none' ? null : s.note),
                          }
                        : s
                    ),
                  }
                : p
            ),
          },
        };
      }),
    })),

  setStepNote: (trackId, pageId, stepId, note) =>
    set((state) => ({
      tracks: state.tracks.map((t) => {
        if (t.id !== trackId) return t;
        return {
          ...t,
          sequencer: {
            ...t.sequencer,
            pages: t.sequencer.pages.map((p) =>
              p.id === pageId
                ? {
                    ...p,
                    steps: p.steps.map((s) =>
                      s.id === stepId ? { ...s, note } : s
                    ),
                  }
                : p
            ),
          },
        };
      }),
    })),

  addParamLock: (trackId, pageId, stepId, param, value) =>
    set((state) => ({
      tracks: state.tracks.map((t) => {
        if (t.id !== trackId) return t;
        return {
          ...t,
          sequencer: {
            ...t.sequencer,
            pages: t.sequencer.pages.map((p) =>
              p.id === pageId
                ? {
                    ...p,
                    steps: p.steps.map((s) =>
                      s.id === stepId
                        ? { ...s, paramLocks: { ...s.paramLocks, [param]: value } }
                        : s
                    ),
                  }
                : p
            ),
          },
        };
      }),
    })),

  removeParamLock: (trackId, pageId, stepId, param) =>
    set((state) => ({
      tracks: state.tracks.map((t) => {
        if (t.id !== trackId) return t;
        const newParamLocks = { ...t.sequencer.pages[pageId].steps[stepId].paramLocks };
        delete newParamLocks[param];
        return {
          ...t,
          sequencer: {
            ...t.sequencer,
            pages: t.sequencer.pages.map((p) =>
              p.id === pageId
                ? {
                    ...p,
                    steps: p.steps.map((s) =>
                      s.id === stepId
                        ? { ...s, paramLocks: newParamLocks }
                        : s
                    ),
                  }
                : p
            ),
          },
        };
      }),
    })),

  setTransport: (transport) =>
    set((state) => ({
      transport: { ...state.transport, ...transport },
    })),

  nextStep: () =>
    set((state) => {
      const { transport, tracks } = state;
      let nextStep = transport.currentStep + 1;
      let nextPage = transport.currentPage;

      // Calculate total steps based on longest track length
      const maxLength = Math.max(...tracks.map((t) => t.sequencer.length));
      const totalSteps = maxLength * 16; // 16 pages

      if (nextStep >= totalSteps) {
        nextStep = 0;
      }
      if (nextStep % 16 === 0) {
        nextPage = Math.floor(nextStep / 16);
      }

      return {
        transport: {
          ...transport,
          currentStep: nextStep,
          currentPage: nextPage,
        },
      };
    }),
}));

// Helper functions
export const PAGE_LABELS: Record<PageType, string> = {
  trig: 'TRIG',
  src: 'SRC',
  fltr: 'FLTR',
  amp: 'AMP',
  fx: 'FX',
  mod: 'MOD',
};

export const TRACK_TYPE_CONFIG: Record<TrackType, { color: string; fxSlots: number }> = {
  audio: { color: '#00ff88', fxSlots: 2 },
  bus: { color: '#00d4ff', fxSlots: 2 },
  send: { color: '#bf5fff', fxSlots: 1 },
  mix: { color: '#ffcc00', fxSlots: 1 },
};
