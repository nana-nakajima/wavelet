import { useWaveletStore } from "~/store/wavelet-store";
import { Encoder } from "./Encoder";

export function TransportBar() {
  const transport = useWaveletStore((s) => s.transport);
  const play = useWaveletStore((s) => s.play);
  const stop = useWaveletStore((s) => s.stop);
  const toggleRecord = useWaveletStore((s) => s.toggleRecord);
  const setTempo = useWaveletStore((s) => s.setTempo);

  return (
    <div className="flex items-center justify-between px-3 py-1.5 bg-chassis-900 border-b border-chassis-700">
      {/* Left: transport buttons */}
      <div className="flex items-center gap-2">
        <button
          onClick={play}
          className={`w-8 h-8 flex items-center justify-center rounded text-sm
            ${transport.playing
              ? "bg-oled-green/20 text-oled-green"
              : "bg-chassis-800 text-chassis-600 hover:text-white/60"
            }`}
        >
          ▶
        </button>
        <button
          onClick={stop}
          className="w-8 h-8 flex items-center justify-center rounded bg-chassis-800 text-chassis-600 hover:text-white text-sm"
        >
          ■
        </button>
        <button
          onClick={toggleRecord}
          className={`w-8 h-8 flex items-center justify-center rounded text-sm
            ${transport.recording
              ? "bg-oled-red/20 text-oled-red"
              : "bg-chassis-800 text-chassis-600 hover:text-white/60"
            }`}
        >
          ●
        </button>
      </div>

      {/* Center: tempo */}
      <div className="flex items-center gap-3">
        <Encoder
          label="BPM"
          value={(transport.tempo - 20) / 280}
          displayValue={`${transport.tempo.toFixed(1)}`}
          onChange={(v) => setTempo(20 + v * 280)}
        />
      </div>

      {/* Right: step position + pattern info */}
      <div className="flex items-center gap-4 text-[10px] font-mono">
        <span className="text-oled-teal tabular-nums">
          STEP {String(transport.currentStep + 1).padStart(2, "0")}/16
        </span>
        <span className="text-chassis-600">
          {transport.playMode.toUpperCase()} {transport.patternBank}:{String(transport.currentPage + 1).padStart(2, "0")}
        </span>
      </div>
    </div>
  );
}
