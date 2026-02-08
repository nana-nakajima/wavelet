import { useCallback } from "react";
import { useWaveletStore } from "~/store/wavelet-store";
import { useLongPress } from "~/hooks/use-long-press";
import type { TrigType } from "~/store/wavelet-store";

function StepButton({ index }: { index: number }) {
  const transport = useWaveletStore((s) => s.transport);
  const activeTrack = useWaveletStore((s) => s.ui.activeTrack);
  const step = useWaveletStore(
    (s) => s.tracks[s.ui.activeTrack].pages[s.transport.currentPage].steps[index]
  );
  const pLockStep = useWaveletStore((s) => s.ui.pLockStep);
  const toggleStep = useWaveletStore((s) => s.toggleStep);
  const startPLockEdit = useWaveletStore((s) => s.startPLockEdit);
  const endPLockEdit = useWaveletStore((s) => s.endPLockEdit);

  const onLongPress = useCallback(() => startPLockEdit(index), [index, startPLockEdit]);
  const onRelease = useCallback(() => endPLockEdit(), [endPLockEdit]);
  const onClick = useCallback(() => toggleStep(index), [index, toggleStep]);

  const { onPointerDown, onPointerUp, onPointerLeave } = useLongPress(
    onLongPress,
    onRelease,
    onClick
  );

  const isPlayhead = transport.playing && transport.currentStep === index;
  const isPLockActive = pLockStep === index;

  const colorClass = getStepColor(step.trigType, isPlayhead, isPLockActive);

  return (
    <button
      onPointerDown={onPointerDown}
      onPointerUp={onPointerUp}
      onPointerLeave={onPointerLeave}
      className={`
        w-10 h-10 rounded-sm border transition-all select-none
        ${colorClass}
        ${isPlayhead ? "step-active ring-1 ring-white" : ""}
        ${isPLockActive ? "animate-pulse" : ""}
      `}
    />
  );
}

function getStepColor(trigType: TrigType, isPlayhead: boolean, isPLock: boolean): string {
  if (isPLock) return "bg-oled-orange/30 border-oled-orange";
  switch (trigType) {
    case "none":
      return isPlayhead
        ? "bg-chassis-700 border-white/40"
        : "bg-chassis-800 border-chassis-700 hover:border-chassis-600";
    case "note":
      return "bg-oled-teal/20 border-oled-teal";
    case "lock":
      return "bg-oled-amber/20 border-oled-amber";
    case "combined":
      return "bg-oled-teal/20 border-oled-teal ring-1 ring-oled-amber/50";
  }
}

export function StepGrid() {
  return (
    <div className="flex items-center gap-1 px-3 py-2">
      {Array.from({ length: 16 }, (_, i) => (
        <StepButton key={i} index={i} />
      ))}
    </div>
  );
}
