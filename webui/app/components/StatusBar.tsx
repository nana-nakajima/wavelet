import { useWaveletStore } from "~/store/wavelet-store";
import { engine } from "~/engine/wavelet-engine";

export function StatusBar() {
  const engineReady = useWaveletStore((s) => s.engineReady);
  const cpuLoad = useWaveletStore((s) => s.engineReadback.cpuLoad);
  const peakL = useWaveletStore((s) => s.engineReadback.peakL);
  const peakR = useWaveletStore((s) => s.engineReadback.peakR);

  const state = engine.state;
  const stateColor =
    state === "running" ? "bg-oled-green" :
    state === "suspended" ? "bg-oled-amber" :
    state === "error" ? "bg-oled-red" :
    "bg-chassis-600";

  const cpuPercent = Math.round(cpuLoad * 100);

  return (
    <div className="flex items-center justify-between px-3 py-0.5 bg-chassis-950 border-t border-chassis-700 text-[8px] font-mono">
      {/* Left: engine state */}
      <div className="flex items-center gap-1.5">
        <div className={`w-1.5 h-1.5 rounded-full ${stateColor}`} />
        <span className="text-chassis-600 uppercase">{state}</span>
      </div>

      {/* Center: CPU load */}
      <div className="flex items-center gap-1.5">
        <span className="text-chassis-600">CPU</span>
        <div className="w-16 h-1.5 bg-chassis-800 rounded-full overflow-hidden">
          <div
            className="h-full rounded-full transition-all duration-150"
            style={{
              width: `${cpuPercent}%`,
              backgroundColor: cpuPercent > 80 ? "#ff2d2d" : cpuPercent > 50 ? "#ffb300" : "#00e5c8",
            }}
          />
        </div>
        <span className="text-chassis-600 tabular-nums w-6 text-right">{cpuPercent}%</span>
      </div>

      {/* Right: peak meters + latency */}
      <div className="flex items-center gap-1.5">
        <span className="text-chassis-600">L</span>
        <div className="w-8 h-1.5 bg-chassis-800 rounded-full overflow-hidden">
          <div
            className="h-full rounded-full transition-all duration-75"
            style={{
              width: `${Math.min(100, peakL * 100)}%`,
              backgroundColor: peakL > 0.9 ? "#ff2d2d" : "#00e5c8",
            }}
          />
        </div>
        <span className="text-chassis-600">R</span>
        <div className="w-8 h-1.5 bg-chassis-800 rounded-full overflow-hidden">
          <div
            className="h-full rounded-full transition-all duration-75"
            style={{
              width: `${Math.min(100, peakR * 100)}%`,
              backgroundColor: peakR > 0.9 ? "#ff2d2d" : "#00e5c8",
            }}
          />
        </div>
      </div>
    </div>
  );
}
