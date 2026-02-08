const NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

export function formatNote(midi: number): string {
  const octave = Math.floor(midi / 12) - 1;
  const name = NOTE_NAMES[midi % 12];
  return `${name}${octave}`;
}

export function formatDb(value: number): string {
  if (value <= 0) return "-inf";
  const db = 20 * Math.log10(value);
  if (db <= -60) return "-inf";
  return `${db >= 0 ? "+" : ""}${db.toFixed(1)}dB`;
}

export function formatHz(value: number, maxHz = 20000): string {
  const hz = value * maxHz;
  if (hz >= 1000) return `${(hz / 1000).toFixed(1)}kHz`;
  return `${Math.round(hz)}Hz`;
}

export function formatPercent(value: number): string {
  return `${Math.round(value * 100)}%`;
}

export function formatBipolar(value: number): string {
  const v = Math.round(value * 100);
  if (v === 0) return "0";
  return v > 0 ? `+${v}` : `${v}`;
}

export function formatBpm(value: number): string {
  return `${value.toFixed(1)}`;
}

export function formatParamValue(param: string, value: number): string {
  if (param === "freq" || param === "lfo1_speed" || param === "lfo2_speed" || param === "lfo_speed") {
    return formatHz(value);
  }
  if (param === "tune" || param === "fine" || param === "pan" || param === "micro_timing") {
    return formatBipolar(value);
  }
  if (param === "note") {
    return formatNote(Math.round(value * 127));
  }
  if (param === "velocity") {
    return `${Math.round(value * 127)}`;
  }
  if (param.includes("send") || param === "level" || param === "volume") {
    return formatDb(value);
  }
  return formatPercent(value);
}
