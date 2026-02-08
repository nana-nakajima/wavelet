import { useEncoderDrag } from "~/hooks/use-encoder-drag";

interface EncoderProps {
  label: string;
  value: number;
  displayValue: string;
  onChange: (v: number) => void;
  fine?: boolean;
  pLockActive?: boolean;
}

export function Encoder({ label, value, displayValue, onChange, fine = false, pLockActive = false }: EncoderProps) {
  const { onPointerDown, onPointerMove, onPointerUp } = useEncoderDrag({
    value,
    onChange,
    fine,
  });

  // Knob rotation: 135° to 45° sweeping clockwise through top (270° range)
  // 0 = 7:30 (lower-left), 0.5 = 12:00 (top), 1 = 4:30 (lower-right)
  const angle = 135 + value * 270;

  return (
    <div className="flex flex-col items-center gap-1 select-none">
      <span className="text-[9px] uppercase tracking-wider text-chassis-600">
        {label}
      </span>
      <svg
        width="44"
        height="44"
        viewBox="0 0 44 44"
        className="cursor-ns-resize"
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
      >
        {/* Outer ring */}
        <circle
          cx="22" cy="22" r="20"
          fill="none"
          stroke={pLockActive ? "#ff6a00" : "#303036"}
          strokeWidth="1.5"
          className={pLockActive ? "animate-pulse" : ""}
        />
        {/* Track arc background */}
        <circle
          cx="22" cy="22" r="17"
          fill="#1a1a1f"
          stroke="#252529"
          strokeWidth="1"
        />
        {/* Tick marks */}
        {[0, 0.25, 0.5, 0.75, 1].map((t) => {
          const a = ((135 + t * 270) * Math.PI) / 180;
          const x1 = 22 + Math.cos(a) * 14;
          const y1 = 22 + Math.sin(a) * 14;
          const x2 = 22 + Math.cos(a) * 17;
          const y2 = 22 + Math.sin(a) * 17;
          return (
            <line
              key={t}
              x1={x1} y1={y1} x2={x2} y2={y2}
              stroke="#303036" strokeWidth="1"
            />
          );
        })}
        {/* Pointer indicator */}
        <line
          x1="22" y1="22"
          x2={22 + Math.cos((angle * Math.PI) / 180) * 13}
          y2={22 + Math.sin((angle * Math.PI) / 180) * 13}
          stroke={pLockActive ? "#ff6a00" : "#00e5c8"}
          strokeWidth="2"
          strokeLinecap="round"
        />
        {/* Center dot */}
        <circle cx="22" cy="22" r="3" fill="#252529" />
      </svg>
      <span className="text-[9px] text-oled-teal tabular-nums">
        {displayValue}
      </span>
    </div>
  );
}
