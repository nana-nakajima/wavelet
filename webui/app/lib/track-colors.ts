import type { TrackType } from "~/store/wavelet-store";

const TRACK_COLORS: Record<TrackType, string> = {
  audio: "#22c55e",
  bus: "#3b82f6",
  send: "#a855f7",
  mix: "#eab308",
};

const TRACK_TW: Record<TrackType, string> = {
  audio: "text-track-audio",
  bus: "text-track-bus",
  send: "text-track-send",
  mix: "text-track-mix",
};

const TRACK_BG_TW: Record<TrackType, string> = {
  audio: "bg-track-audio",
  bus: "bg-track-bus",
  send: "bg-track-send",
  mix: "bg-track-mix",
};

const TRACK_BORDER_TW: Record<TrackType, string> = {
  audio: "border-track-audio",
  bus: "border-track-bus",
  send: "border-track-send",
  mix: "border-track-mix",
};

export function getTrackColor(type: TrackType): string {
  return TRACK_COLORS[type];
}

export function getTrackTw(type: TrackType): string {
  return TRACK_TW[type];
}

export function getTrackBgTw(type: TrackType): string {
  return TRACK_BG_TW[type];
}

export function getTrackBorderTw(type: TrackType): string {
  return TRACK_BORDER_TW[type];
}
