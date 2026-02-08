import { useWaveletStore } from "~/store/wavelet-store";
import type { PageType } from "~/store/wavelet-store";

const PAGES: { id: PageType; label: string }[] = [
  { id: "trig", label: "TRIG" },
  { id: "src", label: "SRC" },
  { id: "fltr", label: "FLTR" },
  { id: "amp", label: "AMP" },
  { id: "fx", label: "FX" },
  { id: "mod", label: "MOD" },
];

export function PageButtons() {
  const activePage = useWaveletStore((s) => s.ui.activePage);
  const setPage = useWaveletStore((s) => s.setPage);

  return (
    <div className="flex gap-1">
      {PAGES.map((p) => (
        <button
          key={p.id}
          onClick={() => setPage(p.id)}
          className={`
            px-2.5 py-1 text-[9px] font-bold uppercase tracking-wider rounded
            transition-colors
            ${activePage === p.id
              ? "bg-oled-teal/20 text-oled-teal border border-oled-teal/40"
              : "bg-chassis-800 text-chassis-600 border border-chassis-700 hover:text-white/60"
            }
          `}
        >
          {p.label}
        </button>
      ))}
    </div>
  );
}
