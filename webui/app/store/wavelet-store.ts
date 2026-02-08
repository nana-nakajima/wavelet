// ==========================================================================
// wavelet-store.ts — Zustand store with SharedArrayBuffer sync
//
// This store is the single source of truth for the UI. It:
//   1. Mirrors the 16-track architecture from the Rust engine
//   2. Writes parameter changes to the SharedArrayBuffer in real-time
//   3. Reads engine-computed state (step position, peaks, CPU) via RAF loop
//   4. Manages sequencer state including P-Lock editing
//   5. Manages UI-only state (selected track, active page, FUNC key, etc.)
// ==========================================================================

import { create } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import { engine, SAB, PARAMS_PER_TRACK } from "~/engine/wavelet-engine";

// ── Types ──

export type TrackType = "audio" | "bus" | "send" | "mix";

export type PageType = "trig" | "src" | "fltr" | "amp" | "fx" | "mod" | "lfo";

export type TrigType = "none" | "note" | "lock" | "combined";

export type TrigCondition =
  | "none" | "fill" | "not_fill" | "pre" | "not_pre"
  | "nei" | "not_nei" | "first" | "not_first"
  | "1:2" | "2:2" | "1:3" | "2:3" | "3:3"
  | "1:4" | "2:4" | "3:4" | "4:4";

export interface ParamLock {
  param: string;
  value: number;
}

export interface StepData {
  id: number;
  trigType: TrigType;
  note: number | null;
  velocity: number;
  paramLocks: ParamLock[];
  microTiming: number;
  retrig: boolean;
  retrigRate: number;
  condition: TrigCondition;
}

export interface SequencerPage {
  id: number;
  steps: StepData[];
  length: number;
  scale: number;
}

export interface FxSlot {
  type: string;
  params: Record<string, number>;
  enabled: boolean;
}

export interface TrackData {
  id: number;
  type: TrackType;
  name: string;
  muted: boolean;
  solo: boolean;
  volume: number;
  pan: number;

  // Current parameter page
  currentPage: PageType;

  // SRC parameters (8 encoders worth)
  srcParams: Record<string, number>;

  // FLTR parameters
  fltrParams: Record<string, number>;

  // AMP parameters
  ampParams: Record<string, number>;

  // FX slots
  fxSlots: FxSlot[];

  // MOD parameters
  modParams: Record<string, number>;

  // LFO parameters
  lfoParams: Record<string, number>;

  // Sequencer (16 pages × 16 steps = 256 steps max)
  pages: SequencerPage[];
  patternLength: number;
}

// ── Encoder parameter mapping per page ──
// Each page exposes 8 parameters to encoders A–H

export const PAGE_ENCODER_MAP: Record<PageType, string[]> = {
  trig: ["note", "velocity", "length", "micro_timing", "retrig_rate", "retrig_vel", "condition", "probability"],
  src:  ["tune", "fine", "decay", "hold", "sweep", "contour", "noise", "level"],
  fltr: ["freq", "reso", "type", "env_depth", "env_decay", "base_width", "drive", "key_track"],
  amp:  ["attack", "hold", "decay", "sustain", "release", "overdrive", "delay_send", "reverb_send"],
  fx:   ["fx1_param1", "fx1_param2", "fx1_param3", "fx1_mix", "fx2_param1", "fx2_param2", "fx2_param3", "fx2_mix"],
  mod:  ["lfo1_speed", "lfo1_depth", "lfo1_dest", "lfo1_wave", "lfo2_speed", "lfo2_depth", "lfo2_dest", "lfo2_wave"],
  lfo:  ["lfo_speed", "lfo_multiply", "lfo_fade", "lfo_dest", "lfo_wave", "lfo_start_phase", "lfo_mode", "lfo_depth"],
};

// ── Transport state ──

export interface TransportState {
  playing: boolean;
  recording: boolean;
  tempo: number;
  currentStep: number;
  currentPage: number;
  patternBank: number;
  playMode: "pattern" | "song";
}

// ── UI-only state ──

export interface UIState {
  activeTrack: number;
  activePage: PageType;
  funcHeld: boolean;
  pLockStep: number | null;  // non-null when long-pressing a step for P-Lock editing
  planarX: number;           // 2D planar macro X (0–1)
  planarY: number;           // 2D planar macro Y (0–1)
}

// ── Engine readback state (written by RAF loop) ──

export interface EngineReadback {
  peakL: number;
  peakR: number;
  cpuLoad: number;
  waveform: Float32Array;
}

// ── Full store shape ──

export interface WaveletStore {
  // Data
  tracks: TrackData[];
  transport: TransportState;
  ui: UIState;
  engineReadback: EngineReadback;
  engineReady: boolean;

  // ── Transport actions ──
  play: () => void;
  stop: () => void;
  toggleRecord: () => void;
  setTempo: (bpm: number) => void;

  // ── Track actions ──
  selectTrack: (id: number) => void;
  setTrackVolume: (track: number, vol: number) => void;
  setTrackPan: (track: number, pan: number) => void;
  toggleMute: (track: number) => void;
  toggleSolo: (track: number) => void;
  setPage: (page: PageType) => void;

  // ── Encoder actions (A–H) ──
  setEncoderValue: (encoderIndex: number, value: number) => void;

  // ── P-Lock actions ──
  startPLockEdit: (stepIndex: number) => void;
  endPLockEdit: () => void;
  setPLockValue: (encoderIndex: number, value: number) => void;
  clearPLock: (stepIndex: number, param?: string) => void;

  // ── Step / Sequencer actions ──
  toggleStep: (stepIndex: number) => void;
  setStepNote: (stepIndex: number, note: number) => void;
  setStepVelocity: (stepIndex: number, velocity: number) => void;
  setStepCondition: (stepIndex: number, condition: TrigCondition) => void;
  setPatternLength: (track: number, length: number) => void;
  setSequencerPage: (pageIndex: number) => void;

  // ── Planar macro ──
  setPlanar: (x: number, y: number) => void;

  // ── FUNC key ──
  setFuncHeld: (held: boolean) => void;

  // ── MIDI ──
  noteOn: (note: number, velocity: number) => void;
  noteOff: (note: number) => void;

  // ── Engine sync ──
  syncFromEngine: () => void;
  setEngineReady: (ready: boolean) => void;
}

// ── Helpers ──

function createDefaultStep(id: number): StepData {
  return {
    id,
    trigType: "none",
    note: null,
    velocity: 100,
    paramLocks: [],
    microTiming: 0,
    retrig: false,
    retrigRate: 4,
    condition: "none",
  };
}

function createDefaultPage(id: number): SequencerPage {
  return {
    id,
    steps: Array.from({ length: 16 }, (_, i) => createDefaultStep(i)),
    length: 16,
    scale: 1.0,
  };
}

function trackTypeForIndex(i: number): TrackType {
  if (i < 8) return "audio";
  if (i < 12) return "bus";
  if (i < 15) return "send";
  return "mix";
}

function trackNameForIndex(i: number): string {
  const type = trackTypeForIndex(i);
  switch (type) {
    case "audio": return `T${i + 1}`;
    case "bus":   return `BUS ${i - 7}`;
    case "send":  return `SND ${i - 11}`;
    case "mix":   return "MIX";
  }
}

function createDefaultTrack(id: number): TrackData {
  return {
    id,
    type: trackTypeForIndex(id),
    name: trackNameForIndex(id),
    muted: false,
    solo: false,
    volume: 0.8,
    pan: 0.0,
    currentPage: "src",
    srcParams: { tune: 0, fine: 0, decay: 0.5, hold: 0, sweep: 0, contour: 0.5, noise: 0, level: 0.8 },
    fltrParams: { freq: 1.0, reso: 0, type: 0, env_depth: 0, env_decay: 0.5, base_width: 0.5, drive: 0, key_track: 0 },
    ampParams: { attack: 0, hold: 0, decay: 0.5, sustain: 0.8, release: 0.3, overdrive: 0, delay_send: 0, reverb_send: 0 },
    fxSlots: [
      { type: "bypass", params: {}, enabled: false },
      { type: "bypass", params: {}, enabled: false },
    ],
    modParams: { lfo1_speed: 0.5, lfo1_depth: 0, lfo1_dest: 0, lfo1_wave: 0, lfo2_speed: 0.5, lfo2_depth: 0, lfo2_dest: 0, lfo2_wave: 0 },
    lfoParams: { lfo_speed: 0.5, lfo_multiply: 1, lfo_fade: 0, lfo_dest: 0, lfo_wave: 0, lfo_start_phase: 0, lfo_mode: 0, lfo_depth: 0 },
    pages: Array.from({ length: 16 }, (_, i) => createDefaultPage(i)),
    patternLength: 16,
  };
}

// ==========================================================================
// Store creation
// ==========================================================================

export const useWaveletStore = create<WaveletStore>()(
  subscribeWithSelector((set, get) => ({
    // ── Initial state ──

    tracks: Array.from({ length: 16 }, (_, i) => createDefaultTrack(i)),

    transport: {
      playing: false,
      recording: false,
      tempo: 120,
      currentStep: 0,
      currentPage: 0,
      patternBank: 0,
      playMode: "pattern",
    },

    ui: {
      activeTrack: 0,
      activePage: "src",
      funcHeld: false,
      pLockStep: null,
      planarX: 0.5,
      planarY: 0.5,
    },

    engineReadback: {
      peakL: 0,
      peakR: 0,
      cpuLoad: 0,
      waveform: new Float32Array(64),
    },

    engineReady: typeof window !== "undefined" && import.meta.env.DEV && new URLSearchParams(window.location.search).has("dev"),

    // ── Transport ──

    play: () => {
      set((s) => ({ transport: { ...s.transport, playing: true } }));
      engine.setTransportPlaying(true);
    },

    stop: () => {
      set((s) => ({ transport: { ...s.transport, playing: false, currentStep: 0 } }));
      engine.setTransportPlaying(false);
    },

    toggleRecord: () => {
      const rec = !get().transport.recording;
      set((s) => ({ transport: { ...s.transport, recording: rec } }));
      engine.setTransportRecording(rec);
    },

    setTempo: (bpm) => {
      const clamped = Math.max(20, Math.min(300, bpm));
      set((s) => ({ transport: { ...s.transport, tempo: clamped } }));
      engine.setTempo(clamped);
    },

    // ── Track selection ──

    selectTrack: (id) => {
      set((s) => ({ ui: { ...s.ui, activeTrack: id } }));
      engine.setActiveTrack(id);
    },

    setTrackVolume: (track, vol) => {
      set((s) => {
        const tracks = [...s.tracks];
        tracks[track] = { ...tracks[track], volume: vol };
        return { tracks };
      });
      engine.setTrackVolume(track, vol);
    },

    setTrackPan: (track, pan) => {
      set((s) => {
        const tracks = [...s.tracks];
        tracks[track] = { ...tracks[track], pan };
        return { tracks };
      });
      engine.setTrackPan(track, pan);
    },

    toggleMute: (track) => {
      set((s) => {
        const tracks = [...s.tracks];
        const muted = !tracks[track].muted;
        tracks[track] = { ...tracks[track], muted };
        return { tracks };
      });
      engine.setTrackMute(track, !get().tracks[track].muted);
    },

    toggleSolo: (track) => {
      set((s) => {
        const tracks = [...s.tracks];
        const solo = !tracks[track].solo;
        tracks[track] = { ...tracks[track], solo };
        return { tracks };
      });
      engine.setTrackSolo(track, !get().tracks[track].solo);
    },

    setPage: (page) => {
      set((s) => ({ ui: { ...s.ui, activePage: page } }));
    },

    // ── Encoder value (writes to current page's param for active track) ──

    setEncoderValue: (encoderIndex, value) => {
      const { ui, tracks } = get();
      const track = tracks[ui.activeTrack];
      const paramNames = PAGE_ENCODER_MAP[ui.activePage];
      const paramName = paramNames?.[encoderIndex];
      if (!paramName) return;

      // If P-Lock editing, write to the step's param locks instead
      if (ui.pLockStep !== null) {
        get().setPLockValue(encoderIndex, value);
        return;
      }

      // Write to the track's page-specific params
      set((s) => {
        const tracks = [...s.tracks];
        const t = { ...tracks[ui.activeTrack] };
        const pageKey = `${ui.activePage}Params` as keyof TrackData;
        if (typeof t[pageKey] === "object" && t[pageKey] !== null) {
          (t[pageKey] as Record<string, number>)[paramName] = value;
        }
        tracks[ui.activeTrack] = t;
        return { tracks };
      });

      // Sync to engine via SharedArrayBuffer
      engine.setTrackParam(ui.activeTrack, encoderIndex, value);
    },

    // ── P-Lock editing ──

    startPLockEdit: (stepIndex) => {
      set((s) => ({ ui: { ...s.ui, pLockStep: stepIndex } }));
    },

    endPLockEdit: () => {
      set((s) => ({ ui: { ...s.ui, pLockStep: null } }));
    },

    setPLockValue: (encoderIndex, value) => {
      const { ui } = get();
      if (ui.pLockStep === null) return;

      const paramNames = PAGE_ENCODER_MAP[ui.activePage];
      const paramName = paramNames?.[encoderIndex];
      if (!paramName) return;

      set((s) => {
        const tracks = [...s.tracks];
        const t = { ...tracks[ui.activeTrack] };
        const pageIdx = s.transport.currentPage;
        const pages = [...t.pages];
        const page = { ...pages[pageIdx] };
        const steps = [...page.steps];
        const step = { ...steps[ui.pLockStep!] };

        // Upsert the param lock
        const locks = [...step.paramLocks];
        const existing = locks.findIndex((l) => l.param === paramName);
        if (existing >= 0) {
          locks[existing] = { param: paramName, value };
        } else {
          locks.push({ param: paramName, value });
        }

        // If step had no trig, promote to lock trig
        if (step.trigType === "none") {
          step.trigType = "lock";
        } else if (step.trigType === "note") {
          step.trigType = "combined";
        }

        step.paramLocks = locks;
        steps[ui.pLockStep!] = step;
        page.steps = steps;
        pages[pageIdx] = page;
        t.pages = pages;
        tracks[ui.activeTrack] = t;
        return { tracks };
      });
    },

    clearPLock: (stepIndex, param) => {
      const { ui, transport } = get();
      set((s) => {
        const tracks = [...s.tracks];
        const t = { ...tracks[ui.activeTrack] };
        const pages = [...t.pages];
        const page = { ...pages[transport.currentPage] };
        const steps = [...page.steps];
        const step = { ...steps[stepIndex] };

        if (param) {
          step.paramLocks = step.paramLocks.filter((l) => l.param !== param);
        } else {
          step.paramLocks = [];
        }

        // Downgrade trig type if no more locks
        if (step.paramLocks.length === 0) {
          if (step.trigType === "lock") step.trigType = "none";
          if (step.trigType === "combined") step.trigType = "note";
        }

        steps[stepIndex] = step;
        page.steps = steps;
        pages[transport.currentPage] = page;
        t.pages = pages;
        tracks[ui.activeTrack] = t;
        return { tracks };
      });
    },

    // ── Step / Sequencer ──

    toggleStep: (stepIndex) => {
      const { ui, transport } = get();
      set((s) => {
        const tracks = [...s.tracks];
        const t = { ...tracks[ui.activeTrack] };
        const pages = [...t.pages];
        const page = { ...pages[transport.currentPage] };
        const steps = [...page.steps];
        const step = { ...steps[stepIndex] };

        if (step.trigType === "none") {
          step.trigType = "note";
          step.note = step.note ?? 60; // Default to C4
        } else {
          step.trigType = "none";
          step.paramLocks = [];
        }

        steps[stepIndex] = step;
        page.steps = steps;
        pages[transport.currentPage] = page;
        t.pages = pages;
        tracks[ui.activeTrack] = t;
        return { tracks };
      });
    },

    setStepNote: (stepIndex, note) => {
      const { ui, transport } = get();
      set((s) => {
        const tracks = [...s.tracks];
        const t = { ...tracks[ui.activeTrack] };
        const pages = [...t.pages];
        const page = { ...pages[transport.currentPage] };
        const steps = [...page.steps];
        steps[stepIndex] = { ...steps[stepIndex], note };
        page.steps = steps;
        pages[transport.currentPage] = page;
        t.pages = pages;
        tracks[ui.activeTrack] = t;
        return { tracks };
      });
    },

    setStepVelocity: (stepIndex, velocity) => {
      const { ui, transport } = get();
      set((s) => {
        const tracks = [...s.tracks];
        const t = { ...tracks[ui.activeTrack] };
        const pages = [...t.pages];
        const page = { ...pages[transport.currentPage] };
        const steps = [...page.steps];
        steps[stepIndex] = { ...steps[stepIndex], velocity };
        page.steps = steps;
        pages[transport.currentPage] = page;
        t.pages = pages;
        tracks[ui.activeTrack] = t;
        return { tracks };
      });
    },

    setStepCondition: (stepIndex, condition) => {
      const { ui, transport } = get();
      set((s) => {
        const tracks = [...s.tracks];
        const t = { ...tracks[ui.activeTrack] };
        const pages = [...t.pages];
        const page = { ...pages[transport.currentPage] };
        const steps = [...page.steps];
        steps[stepIndex] = { ...steps[stepIndex], condition };
        page.steps = steps;
        pages[transport.currentPage] = page;
        t.pages = pages;
        tracks[ui.activeTrack] = t;
        return { tracks };
      });
    },

    setPatternLength: (track, length) => {
      set((s) => {
        const tracks = [...s.tracks];
        tracks[track] = { ...tracks[track], patternLength: length };
        return { tracks };
      });
    },

    setSequencerPage: (pageIndex) => {
      set((s) => ({
        transport: { ...s.transport, currentPage: pageIndex },
      }));
    },

    // ── Planar macro ──

    setPlanar: (x, y) => {
      set((s) => ({ ui: { ...s.ui, planarX: x, planarY: y } }));
      // Map planar to two track params (indices 6 and 7 by convention)
      const { ui } = get();
      engine.setTrackParam(ui.activeTrack, 6, x);
      engine.setTrackParam(ui.activeTrack, 7, y);
    },

    // ── FUNC key ──

    setFuncHeld: (held) => {
      set((s) => ({ ui: { ...s.ui, funcHeld: held } }));
    },

    // ── MIDI ──

    noteOn: (note, velocity) => {
      const { ui } = get();
      engine.noteOn(note, velocity / 127, ui.activeTrack);
    },

    noteOff: (note) => {
      const { ui } = get();
      engine.noteOff(note, ui.activeTrack);
    },

    // ── Engine sync (called from RAF loop) ──

    syncFromEngine: () => {
      const pb = engine.paramBlock;
      if (!pb) return;

      const [peakL, peakR] = engine.getPeakLevels();
      const cpuLoad = engine.getCpuLoad();
      const currentStep = engine.getCurrentStep();
      const waveform = engine.getWaveformData();

      set((s) => ({
        transport: { ...s.transport, currentStep: Math.floor(currentStep) },
        engineReadback: { peakL, peakR, cpuLoad, waveform },
      }));
    },

    setEngineReady: (ready) => {
      set({ engineReady: ready });
    },
  }))
);

