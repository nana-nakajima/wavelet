import { TrackStrip } from "./TrackStrip";

export function TrackStrips() {
  return (
    <div className="flex bg-chassis-900 border-l border-chassis-700 overflow-x-auto overflow-y-hidden">
      {Array.from({ length: 16 }, (_, i) => (
        <TrackStrip key={i} index={i} />
      ))}
    </div>
  );
}
