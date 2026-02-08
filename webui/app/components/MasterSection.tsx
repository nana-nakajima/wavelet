import { useWaveletStore } from "~/store/wavelet-store";
import { Encoder } from "./Encoder";
import { formatDb } from "~/lib/format";
import { engine } from "~/engine/wavelet-engine";

export function MasterSection() {
  const peakL = useWaveletStore((s) => s.engineReadback.peakL);
  const peakR = useWaveletStore((s) => s.engineReadback.peakR);

  // Master volume reads from engine directly
  const masterVol = engine.paramBlock?.[3] ?? 0.8;

  return (
    <div className="flex items-center gap-3">
      <Encoder
        label="MASTER"
        value={masterVol}
        displayValue={formatDb(masterVol)}
        onChange={(v) => engine.setMasterVolume(v)}
      />

      {/* Stereo peak meter */}
      <div className="flex gap-0.5 h-[44px] items-end">
        <div className="w-1.5 h-full bg-chassis-800 rounded-full overflow-hidden relative">
          <div
            className="absolute bottom-0 w-full rounded-full transition-all duration-75"
            style={{
              height: `${Math.min(1, peakL) * 100}%`,
              backgroundColor: peakL > 0.9 ? "#ff2d2d" : "#00e5c8",
            }}
          />
        </div>
        <div className="w-1.5 h-full bg-chassis-800 rounded-full overflow-hidden relative">
          <div
            className="absolute bottom-0 w-full rounded-full transition-all duration-75"
            style={{
              height: `${Math.min(1, peakR) * 100}%`,
              backgroundColor: peakR > 0.9 ? "#ff2d2d" : "#00e5c8",
            }}
          />
        </div>
      </div>
    </div>
  );
}
