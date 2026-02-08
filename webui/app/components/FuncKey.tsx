import { useWaveletStore } from "~/store/wavelet-store";

export function FuncKey() {
  const funcHeld = useWaveletStore((s) => s.ui.funcHeld);
  const setFuncHeld = useWaveletStore((s) => s.setFuncHeld);

  return (
    <button
      onPointerDown={() => setFuncHeld(true)}
      onPointerUp={() => setFuncHeld(false)}
      onPointerLeave={() => setFuncHeld(false)}
      className={`
        px-3 py-1 text-[9px] font-bold uppercase tracking-wider rounded
        transition-colors select-none
        ${funcHeld
          ? "bg-oled-orange/20 text-oled-orange border border-oled-orange/40"
          : "bg-chassis-800 text-chassis-600 border border-chassis-700 hover:text-white/60"
        }
      `}
    >
      FUNC
    </button>
  );
}
