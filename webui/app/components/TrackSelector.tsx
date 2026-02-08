import { useWaveletStore } from "~/store/wavelet-store";
import { getTrackBgTw, getTrackTw } from "~/lib/track-colors";
import type { TrackType } from "~/store/wavelet-store";

const GROUPS: { label: string; type: TrackType; start: number; count: number }[] = [
  { label: "AUDIO", type: "audio", start: 0, count: 8 },
  { label: "BUS", type: "bus", start: 8, count: 4 },
  { label: "SEND", type: "send", start: 12, count: 3 },
  { label: "MIX", type: "mix", start: 15, count: 1 },
];

export function TrackSelector() {
  const activeTrack = useWaveletStore((s) => s.ui.activeTrack);
  const tracks = useWaveletStore((s) => s.tracks);
  const selectTrack = useWaveletStore((s) => s.selectTrack);

  return (
    <div className="flex flex-col w-14 bg-chassis-900 border-r border-chassis-700 overflow-y-auto">
      {GROUPS.map((group) => (
        <div key={group.type} className="flex flex-col">
          <div className={`px-1 py-0.5 text-[7px] uppercase tracking-wider ${getTrackTw(group.type)} opacity-60`}>
            {group.label}
          </div>
          {Array.from({ length: group.count }, (_, i) => {
            const idx = group.start + i;
            const track = tracks[idx];
            const isActive = activeTrack === idx;
            return (
              <button
                key={idx}
                onClick={() => selectTrack(idx)}
                className={`
                  px-1 py-1 text-[9px] text-left truncate transition-colors
                  ${isActive
                    ? `${getTrackBgTw(group.type)}/20 text-white border-l-2 ${
                        group.type === "audio" ? "border-track-audio" :
                        group.type === "bus" ? "border-track-bus" :
                        group.type === "send" ? "border-track-send" :
                        "border-track-mix"
                      }`
                    : "text-chassis-600 border-l-2 border-transparent hover:text-white/60"
                  }
                `}
              >
                {track.name}
              </button>
            );
          })}
        </div>
      ))}
    </div>
  );
}
