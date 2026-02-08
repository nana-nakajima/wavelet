import { useWaveletStore, PAGE_ENCODER_MAP } from "~/store/wavelet-store";
import { formatParamValue } from "~/lib/format";
import { Encoder } from "./Encoder";

export function EncoderSection() {
  const activePage = useWaveletStore((s) => s.ui.activePage);
  const activeTrack = useWaveletStore((s) => s.ui.activeTrack);
  const tracks = useWaveletStore((s) => s.tracks);
  const funcHeld = useWaveletStore((s) => s.ui.funcHeld);
  const pLockStep = useWaveletStore((s) => s.ui.pLockStep);
  const setEncoderValue = useWaveletStore((s) => s.setEncoderValue);

  const track = tracks[activeTrack];
  const paramNames = PAGE_ENCODER_MAP[activePage];

  // Get current values from the track's page params
  const pageKey = `${activePage}Params` as keyof typeof track;
  const pageParams = (track[pageKey] as Record<string, number>) ?? {};

  return (
    <div className="flex items-center gap-3">
      {paramNames.map((param, i) => {
        const value = pageParams[param] ?? 0;
        return (
          <Encoder
            key={`${activePage}-${i}`}
            label={param.replace(/_/g, " ")}
            value={value}
            displayValue={formatParamValue(param, value)}
            onChange={(v) => setEncoderValue(i, v)}
            fine={funcHeld}
            pLockActive={pLockStep !== null}
          />
        );
      })}
    </div>
  );
}
