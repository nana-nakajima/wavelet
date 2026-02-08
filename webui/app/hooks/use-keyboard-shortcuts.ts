import { useEffect } from "react";
import { useWaveletStore } from "~/store/wavelet-store";

// QWERTY → MIDI note mapping (C3 = 48)
const QWERTY_MAP: Record<string, number> = {
  a: 48, w: 49, s: 50, e: 51, d: 52,
  f: 53, t: 54, g: 55, y: 56, h: 57,
  u: 58, j: 59, k: 60, o: 61, l: 62,
  p: 63,
};

export function useKeyboardShortcuts() {
  useEffect(() => {
    const held = new Set<string>();

    function handleKeyDown(e: KeyboardEvent) {
      if (e.repeat) return;
      const key = e.key.toLowerCase();

      // FUNC key (Shift)
      if (e.key === "Shift") {
        useWaveletStore.getState().setFuncHeld(true);
        return;
      }

      // Transport
      if (key === " ") {
        e.preventDefault();
        const s = useWaveletStore.getState();
        s.transport.playing ? s.stop() : s.play();
        return;
      }

      // QWERTY piano
      const note = QWERTY_MAP[key];
      if (note !== undefined && !held.has(key)) {
        held.add(key);
        useWaveletStore.getState().noteOn(note, 100);
      }
    }

    function handleKeyUp(e: KeyboardEvent) {
      const key = e.key.toLowerCase();

      if (e.key === "Shift") {
        useWaveletStore.getState().setFuncHeld(false);
        return;
      }

      const note = QWERTY_MAP[key];
      if (note !== undefined) {
        held.delete(key);
        useWaveletStore.getState().noteOff(note);
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
    };
  }, []);
}
