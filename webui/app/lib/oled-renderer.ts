// oled-renderer.ts — Canvas 2D rendering for the OLED screen
//
// Logical resolution: 128×64 pixels, rendered at 4× scale (512×256 canvas).
// Uses a simple 3×5 pixel font for the dot-matrix aesthetic.

export const OLED_TEAL = "#00e5c8";
export const OLED_ORANGE = "#ff6a00";
export const OLED_DIM = "#333333";
export const OLED_WHITE = "#ffffff";

const SCALE = 4;
const W = 128;
const H = 64;

export type OledCtx = CanvasRenderingContext2D;

export function initOledCanvas(canvas: HTMLCanvasElement): OledCtx {
  canvas.width = W * SCALE;
  canvas.height = H * SCALE;
  const ctx = canvas.getContext("2d")!;
  ctx.imageSmoothingEnabled = false;
  return ctx;
}

export function clearScreen(ctx: OledCtx) {
  ctx.fillStyle = "#000000";
  ctx.fillRect(0, 0, W * SCALE, H * SCALE);
}

export function drawDotMatrixText(
  ctx: OledCtx,
  x: number,
  y: number,
  text: string,
  color: string = OLED_TEAL,
  size: number = 1
) {
  const charW = 4 * size;
  const pixelSize = size * SCALE;
  ctx.fillStyle = color;

  for (let ci = 0; ci < text.length; ci++) {
    const ch = text[ci];
    const pattern = FONT_3X5[ch] ?? FONT_3X5["?"];
    if (!pattern) continue;

    for (let row = 0; row < 5; row++) {
      for (let col = 0; col < 3; col++) {
        if (pattern[row] & (1 << (2 - col))) {
          ctx.fillRect(
            (x + ci * charW + col * size) * SCALE,
            (y + row * size) * SCALE,
            pixelSize,
            pixelSize
          );
        }
      }
    }
  }
}

export function drawWaveform(
  ctx: OledCtx,
  data: Float32Array,
  x: number,
  y: number,
  w: number,
  h: number,
  color: string = OLED_TEAL
) {
  const samples = data.length;
  if (samples === 0) return;
  const midY = y + h / 2;
  ctx.fillStyle = color;

  for (let i = 0; i < w; i++) {
    const si = Math.floor((i / w) * samples);
    const val = data[si] ?? 0;
    const py = midY - val * (h / 2);
    ctx.fillRect((x + i) * SCALE, py * SCALE, SCALE, SCALE);
  }
}

export function drawParamBar(
  ctx: OledCtx,
  x: number,
  y: number,
  w: number,
  h: number,
  value: number,
  color: string = OLED_TEAL
) {
  ctx.fillStyle = OLED_DIM;
  ctx.fillRect(x * SCALE, y * SCALE, w * SCALE, h * SCALE);
  const fillW = Math.round(w * Math.max(0, Math.min(1, value)));
  if (fillW > 0) {
    ctx.fillStyle = color;
    ctx.fillRect(x * SCALE, y * SCALE, fillW * SCALE, h * SCALE);
  }
}

export function drawPageIndicator(
  ctx: OledCtx,
  x: number,
  y: number,
  pages: string[],
  activeIndex: number
) {
  for (let i = 0; i < pages.length; i++) {
    const px = x + i * 5;
    ctx.fillStyle = i === activeIndex ? OLED_TEAL : OLED_DIM;
    ctx.fillRect(px * SCALE, y * SCALE, 3 * SCALE, 3 * SCALE);
  }
}

export function drawEnvelope(
  ctx: OledCtx,
  x: number,
  y: number,
  w: number,
  h: number,
  attack: number,
  decay: number,
  sustain: number,
  release: number,
  color: string = OLED_TEAL
) {
  const totalTime = attack + decay + 0.3 + release;
  const scaleX = (t: number) => x + (t / totalTime) * w;

  const aEnd = scaleX(attack);
  drawLine(ctx, x, y + h, aEnd, y, color);

  const dEnd = scaleX(attack + decay);
  const susY = y + h * (1 - sustain);
  drawLine(ctx, aEnd, y, dEnd, susY, color);

  const sEnd = scaleX(attack + decay + 0.3);
  drawLine(ctx, dEnd, susY, sEnd, susY, color);

  const rEnd = scaleX(totalTime);
  drawLine(ctx, sEnd, susY, rEnd, y + h, color);
}

function drawLine(
  ctx: OledCtx,
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  color: string
) {
  ctx.fillStyle = color;
  const steps = Math.max(Math.abs(x2 - x1), Math.abs(y2 - y1), 1);
  for (let i = 0; i <= steps; i++) {
    const t = i / steps;
    const px = Math.round(x1 + (x2 - x1) * t);
    const py = Math.round(y1 + (y2 - y1) * t);
    ctx.fillRect(px * SCALE, py * SCALE, SCALE, SCALE);
  }
}

// Minimal 3×5 pixel font (each row is a 3-bit bitmask)
const FONT_3X5: Record<string, number[]> = {
  " ": [0, 0, 0, 0, 0],
  "A": [0b010, 0b101, 0b111, 0b101, 0b101],
  "B": [0b110, 0b101, 0b110, 0b101, 0b110],
  "C": [0b011, 0b100, 0b100, 0b100, 0b011],
  "D": [0b110, 0b101, 0b101, 0b101, 0b110],
  "E": [0b111, 0b100, 0b110, 0b100, 0b111],
  "F": [0b111, 0b100, 0b110, 0b100, 0b100],
  "G": [0b011, 0b100, 0b101, 0b101, 0b011],
  "H": [0b101, 0b101, 0b111, 0b101, 0b101],
  "I": [0b111, 0b010, 0b010, 0b010, 0b111],
  "J": [0b001, 0b001, 0b001, 0b101, 0b010],
  "K": [0b101, 0b101, 0b110, 0b101, 0b101],
  "L": [0b100, 0b100, 0b100, 0b100, 0b111],
  "M": [0b101, 0b111, 0b111, 0b101, 0b101],
  "N": [0b101, 0b111, 0b111, 0b101, 0b101],
  "O": [0b010, 0b101, 0b101, 0b101, 0b010],
  "P": [0b110, 0b101, 0b110, 0b100, 0b100],
  "Q": [0b010, 0b101, 0b101, 0b110, 0b011],
  "R": [0b110, 0b101, 0b110, 0b101, 0b101],
  "S": [0b011, 0b100, 0b010, 0b001, 0b110],
  "T": [0b111, 0b010, 0b010, 0b010, 0b010],
  "U": [0b101, 0b101, 0b101, 0b101, 0b010],
  "V": [0b101, 0b101, 0b101, 0b010, 0b010],
  "W": [0b101, 0b101, 0b111, 0b111, 0b101],
  "X": [0b101, 0b101, 0b010, 0b101, 0b101],
  "Y": [0b101, 0b101, 0b010, 0b010, 0b010],
  "Z": [0b111, 0b001, 0b010, 0b100, 0b111],
  "0": [0b010, 0b101, 0b101, 0b101, 0b010],
  "1": [0b010, 0b110, 0b010, 0b010, 0b111],
  "2": [0b110, 0b001, 0b010, 0b100, 0b111],
  "3": [0b110, 0b001, 0b010, 0b001, 0b110],
  "4": [0b101, 0b101, 0b111, 0b001, 0b001],
  "5": [0b111, 0b100, 0b110, 0b001, 0b110],
  "6": [0b011, 0b100, 0b110, 0b101, 0b010],
  "7": [0b111, 0b001, 0b010, 0b010, 0b010],
  "8": [0b010, 0b101, 0b010, 0b101, 0b010],
  "9": [0b010, 0b101, 0b011, 0b001, 0b110],
  ".": [0, 0, 0, 0, 0b010],
  ":": [0, 0b010, 0, 0b010, 0],
  "-": [0, 0, 0b111, 0, 0],
  "+": [0, 0b010, 0b111, 0b010, 0],
  "/": [0b001, 0b001, 0b010, 0b100, 0b100],
  "%": [0b101, 0b001, 0b010, 0b100, 0b101],
  "#": [0b101, 0b111, 0b101, 0b111, 0b101],
  "?": [0b110, 0b001, 0b010, 0, 0b010],
  "!": [0b010, 0b010, 0b010, 0, 0b010],
  "(": [0b010, 0b100, 0b100, 0b100, 0b010],
  ")": [0b010, 0b001, 0b001, 0b001, 0b010],
  "_": [0, 0, 0, 0, 0b111],
  "=": [0, 0b111, 0, 0b111, 0],
  ">": [0b100, 0b010, 0b001, 0b010, 0b100],
  "<": [0b001, 0b010, 0b100, 0b010, 0b001],
};
