import { useWaveletStore } from "~/store/wavelet-store";

const FX_TYPES = ["bypass", "delay", "reverb", "chorus", "distortion", "phaser", "flanger", "compressor"];

export function FxSlotPanel() {
  const activeTrack = useWaveletStore((s) => s.ui.activeTrack);
  const fxSlots = useWaveletStore((s) => s.tracks[s.ui.activeTrack].fxSlots);

  return (
    <div className="flex flex-col gap-1.5 p-2">
      {fxSlots.map((slot, i) => (
        <div key={i} className="flex items-center gap-2 bg-chassis-800 rounded px-2 py-1.5">
          {/* FX type label */}
          <button
            className="text-[9px] uppercase tracking-wider text-oled-teal w-16 text-left truncate hover:text-white transition-colors"
            title="Click to cycle FX type"
          >
            {slot.type}
          </button>

          {/* Bypass toggle */}
          <button
            className={`w-4 h-4 rounded-sm text-[6px] font-bold flex items-center justify-center
              ${slot.enabled
                ? "bg-oled-green/20 text-oled-green"
                : "bg-chassis-700 text-chassis-600"
              }`}
          >
            {slot.enabled ? "ON" : "—"}
          </button>

          {/* 4 mini param bars */}
          <div className="flex gap-1 flex-1">
            {[0, 1, 2, 3].map((pi) => {
              const paramKeys = Object.keys(slot.params);
              const val = paramKeys[pi] ? slot.params[paramKeys[pi]] : 0;
              return (
                <div key={pi} className="flex-1 h-2 bg-chassis-700 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-oled-teal/40 rounded-full"
                    style={{ width: `${(val ?? 0) * 100}%` }}
                  />
                </div>
              );
            })}
          </div>
        </div>
      ))}
    </div>
  );
}
