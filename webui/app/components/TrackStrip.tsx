import { useWaveletStore } from "~/store/wavelet-store";
import { getTrackColor, getTrackBorderTw } from "~/lib/track-colors";

interface TrackStripProps {
  index: number;
}

export function TrackStrip({ index }: TrackStripProps) {
  const track = useWaveletStore((s) => s.tracks[index]);
  const activeTrack = useWaveletStore((s) => s.ui.activeTrack);
  const peakL = useWaveletStore((s) => s.engineReadback.peakL);
  const selectTrack = useWaveletStore((s) => s.selectTrack);
  const toggleMute = useWaveletStore((s) => s.toggleMute);
  const toggleSolo = useWaveletStore((s) => s.toggleSolo);

  const isActive = activeTrack === index;
  const color = getTrackColor(track.type);

  // Approximate per-track VU from master peak (simplified)
  const vuLevel = track.muted ? 0 : Math.min(1, peakL * track.volume);

  return (
    <div
      onClick={() => selectTrack(index)}
      className={`
        flex flex-col items-center w-12 py-1 gap-1 cursor-pointer
        border-l-2 transition-colors
        ${isActive ? getTrackBorderTw(track.type) : "border-transparent"}
        ${isActive ? "bg-white/[0.03]" : "hover:bg-white/[0.02]"}
      `}
    >
      {/* Track number badge */}
      <div
        className="w-6 h-4 rounded-sm flex items-center justify-center text-[7px] font-bold text-black"
        style={{ backgroundColor: color }}
      >
        {index + 1}
      </div>

      {/* VU meter bar */}
      <div className="w-1.5 flex-1 min-h-[20px] bg-chassis-800 rounded-full overflow-hidden relative">
        <div
          className="absolute bottom-0 w-full rounded-full transition-all duration-75"
          style={{
            height: `${vuLevel * 100}%`,
            backgroundColor: vuLevel > 0.9 ? "#ff2d2d" : color,
          }}
        />
      </div>

      {/* Volume indicator */}
      <span className="text-[7px] text-chassis-600 tabular-nums">
        {Math.round(track.volume * 100)}
      </span>

      {/* Mute / Solo */}
      <div className="flex gap-0.5">
        <button
          onClick={(e) => { e.stopPropagation(); toggleMute(index); }}
          className={`w-4 h-4 text-[6px] font-bold rounded-sm flex items-center justify-center
            ${track.muted ? "bg-oled-red/30 text-oled-red" : "bg-chassis-800 text-chassis-600 hover:text-white/60"}`}
        >
          M
        </button>
        <button
          onClick={(e) => { e.stopPropagation(); toggleSolo(index); }}
          className={`w-4 h-4 text-[6px] font-bold rounded-sm flex items-center justify-center
            ${track.solo ? "bg-oled-amber/30 text-oled-amber" : "bg-chassis-800 text-chassis-600 hover:text-white/60"}`}
        >
          S
        </button>
      </div>
    </div>
  );
}
