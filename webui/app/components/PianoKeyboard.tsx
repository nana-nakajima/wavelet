import { useCallback, useRef } from "react";
import { useWaveletStore } from "~/store/wavelet-store";
import { getTrackColor } from "~/lib/track-colors";

// 2 octaves: C3 (48) to B4 (71)
const OCTAVE_START = 48;
const TOTAL_KEYS = 24;

interface KeyDef {
  note: number;
  isBlack: boolean;
  x: number;
  w: number;
}

function buildKeys(): KeyDef[] {
  const keys: KeyDef[] = [];
  const blackPattern = [1, 3, 6, 8, 10]; // semitone offsets within octave
  let whiteIndex = 0;

  for (let i = 0; i < TOTAL_KEYS; i++) {
    const note = OCTAVE_START + i;
    const semitone = i % 12;
    const isBlack = blackPattern.includes(semitone);

    if (!isBlack) {
      keys.push({ note, isBlack: false, x: whiteIndex, w: 1 });
      whiteIndex++;
    }
  }

  // Add black keys on top
  whiteIndex = 0;
  for (let i = 0; i < TOTAL_KEYS; i++) {
    const note = OCTAVE_START + i;
    const semitone = i % 12;
    const isBlack = blackPattern.includes(semitone);

    if (isBlack) {
      keys.push({ note, isBlack: true, x: whiteIndex - 0.3, w: 0.6 });
    } else {
      whiteIndex++;
    }
  }

  return keys;
}

const KEY_DEFS = buildKeys();
const WHITE_COUNT = KEY_DEFS.filter((k) => !k.isBlack).length;

export function PianoKeyboard() {
  const noteOn = useWaveletStore((s) => s.noteOn);
  const noteOff = useWaveletStore((s) => s.noteOff);
  const trackType = useWaveletStore((s) => s.tracks[s.ui.activeTrack].type);
  const activeNotes = useRef(new Set<number>());

  const trackColor = getTrackColor(trackType);

  const handlePointerDown = useCallback(
    (note: number) => {
      if (!activeNotes.current.has(note)) {
        activeNotes.current.add(note);
        noteOn(note, 100);
      }
    },
    [noteOn]
  );

  const handlePointerUp = useCallback(
    (note: number) => {
      activeNotes.current.delete(note);
      noteOff(note);
    },
    [noteOff]
  );

  const whiteWidth = 100 / WHITE_COUNT;

  return (
    <div className="relative w-full h-full min-h-[60px] select-none">
      {KEY_DEFS.map((k) => {
        const left = `${k.x * whiteWidth}%`;
        const width = `${k.w * whiteWidth}%`;

        if (k.isBlack) {
          return (
            <div
              key={k.note}
              onPointerDown={() => handlePointerDown(k.note)}
              onPointerUp={() => handlePointerUp(k.note)}
              onPointerLeave={() => handlePointerUp(k.note)}
              className="absolute top-0 h-[58%] rounded-b z-10 cursor-pointer
                bg-chassis-950 border border-chassis-700 hover:bg-chassis-800
                active:bg-oled-teal/30"
              style={{ left, width }}
            />
          );
        }

        return (
          <div
            key={k.note}
            onPointerDown={() => handlePointerDown(k.note)}
            onPointerUp={() => handlePointerUp(k.note)}
            onPointerLeave={() => handlePointerUp(k.note)}
            className="absolute top-0 h-full cursor-pointer
              bg-chassis-800 border-r border-chassis-700 hover:bg-chassis-700
              active:bg-oled-teal/20"
            style={{ left, width }}
          >
            {k.note % 12 === 0 && (
              <span className="absolute bottom-1 left-1/2 -translate-x-1/2 text-[8px] text-chassis-600">
                C{Math.floor(k.note / 12) - 1}
              </span>
            )}
          </div>
        );
      })}
    </div>
  );
}
