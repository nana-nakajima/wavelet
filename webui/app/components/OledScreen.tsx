import { useEffect, useRef } from "react";
import { useWaveletStore, PAGE_ENCODER_MAP } from "~/store/wavelet-store";
import type { PageType, TrackData, EngineReadback } from "~/store/wavelet-store";
import {
  initOledCanvas,
  clearScreen,
  drawDotMatrixText,
  drawWaveform,
  drawParamBar,
  drawPageIndicator,
  drawEnvelope,
  OLED_TEAL,
  OLED_ORANGE,
  OLED_DIM,
  type OledCtx,
} from "~/lib/oled-renderer";
import { formatParamValue } from "~/lib/format";

const PAGES: PageType[] = ["trig", "src", "fltr", "amp", "fx", "mod"];

export function OledScreen() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const ctxRef = useRef<OledCtx | null>(null);
  const rafRef = useRef<number>(0);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    ctxRef.current = initOledCanvas(canvas);
    let destroyed = false;

    function render() {
      if (destroyed || !ctxRef.current) return;
      const ctx = ctxRef.current;
      const state = useWaveletStore.getState();
      const { ui, tracks, transport, engineReadback } = state;
      const track = tracks[ui.activeTrack];
      const pageIndex = PAGES.indexOf(ui.activePage);

      clearScreen(ctx);
      drawDotMatrixText(ctx, 1, 1, track.name, OLED_TEAL, 1);
      drawPageIndicator(ctx, 40, 1, PAGES, pageIndex);
      if (ui.pLockStep !== null) {
        drawDotMatrixText(ctx, 90, 1, "P-LOCK", OLED_ORANGE, 1);
      }
      if (transport.playing) {
        drawDotMatrixText(ctx, 110, 1, String(transport.currentStep + 1).padStart(2, "0"), OLED_DIM, 1);
      }
      renderPage(ctx, ui.activePage, track, engineReadback);
      rafRef.current = requestAnimationFrame(render);
    }

    rafRef.current = requestAnimationFrame(render);
    return () => {
      destroyed = true;
      cancelAnimationFrame(rafRef.current);
    };
  }, []);

  return (
    <div className="flex items-center justify-center p-2 bg-black">
      <canvas
        ref={canvasRef}
        className="w-full max-w-[512px] h-auto"
        style={{ imageRendering: "pixelated" }}
      />
    </div>
  );
}

function renderPage(ctx: OledCtx, page: PageType, track: TrackData, readback: EngineReadback) {
  const paramNames = PAGE_ENCODER_MAP[page];

  switch (page) {
    case "src": {
      drawWaveform(ctx, readback.waveform, 2, 10, 60, 24, OLED_TEAL);
      const p = track.srcParams;
      drawDotMatrixText(ctx, 66, 10, "TUNE " + formatParamValue("tune", p.tune), OLED_TEAL, 1);
      drawDotMatrixText(ctx, 66, 18, "FINE " + formatParamValue("fine", p.fine), OLED_TEAL, 1);
      drawDotMatrixText(ctx, 66, 26, "LVL  " + formatParamValue("level", p.level), OLED_TEAL, 1);
      for (let i = 0; i < 8; i++) {
        drawParamBar(ctx, 2 + i * 16, 40, 14, 3, p[paramNames[i]] ?? 0, OLED_TEAL);
        drawDotMatrixText(ctx, 2 + i * 16, 45, paramNames[i].slice(0, 3).toUpperCase(), OLED_DIM, 1);
      }
      break;
    }
    case "fltr": {
      const p = track.fltrParams;
      const freq = p.freq ?? 0.5;
      const reso = p.reso ?? 0;
      for (let x = 0; x < 60; x++) {
        const nx = x / 60;
        const dist = Math.abs(nx - freq);
        const peak = reso * 0.8 * Math.exp(-dist * 10);
        const rolloff = nx > freq ? Math.exp(-(nx - freq) * 8) : 1;
        const y = (rolloff + peak) * 0.8;
        const py = 34 - y * 24;
        ctx.fillStyle = OLED_TEAL;
        ctx.fillRect((2 + x) * 4, py * 4, 4, 4);
      }
      drawDotMatrixText(ctx, 66, 10, "FREQ " + formatParamValue("freq", p.freq), OLED_TEAL, 1);
      drawDotMatrixText(ctx, 66, 18, "RESO " + formatParamValue("reso", p.reso), OLED_TEAL, 1);
      for (let i = 0; i < 8; i++) drawParamBar(ctx, 2 + i * 16, 40, 14, 3, p[paramNames[i]] ?? 0, OLED_TEAL);
      break;
    }
    case "amp": {
      const p = track.ampParams;
      drawEnvelope(ctx, 2, 10, 60, 24, p.attack, p.decay, p.sustain, p.release, OLED_TEAL);
      drawDotMatrixText(ctx, 66, 10, "A " + formatParamValue("attack", p.attack), OLED_TEAL, 1);
      drawDotMatrixText(ctx, 66, 18, "D " + formatParamValue("decay", p.decay), OLED_TEAL, 1);
      drawDotMatrixText(ctx, 66, 26, "S " + formatParamValue("sustain", p.sustain), OLED_TEAL, 1);
      drawDotMatrixText(ctx, 96, 10, "R " + formatParamValue("release", p.release), OLED_TEAL, 1);
      for (let i = 0; i < 8; i++) drawParamBar(ctx, 2 + i * 16, 40, 14, 3, p[paramNames[i]] ?? 0, OLED_TEAL);
      break;
    }
    case "fx": {
      drawDotMatrixText(ctx, 2, 10, "FX1 " + track.fxSlots[0].type.toUpperCase(), OLED_TEAL, 1);
      drawDotMatrixText(ctx, 2, 20, "FX2 " + track.fxSlots[1].type.toUpperCase(), OLED_TEAL, 1);
      for (let i = 0; i < 8; i++) drawParamBar(ctx, 2 + i * 16, 40, 14, 3, 0, OLED_TEAL);
      break;
    }
    case "mod": {
      const p = track.modParams;
      const speed = p.lfo1_speed ?? 0.5;
      for (let x = 0; x < 60; x++) {
        const val = Math.sin((x / 60) * Math.PI * 2 * (1 + speed * 4));
        const py = 22 - val * 10;
        ctx.fillStyle = OLED_TEAL;
        ctx.fillRect((2 + x) * 4, py * 4, 4, 4);
      }
      drawDotMatrixText(ctx, 66, 10, "SPD " + formatParamValue("lfo1_speed", p.lfo1_speed), OLED_TEAL, 1);
      drawDotMatrixText(ctx, 66, 18, "DPT " + formatParamValue("lfo1_depth", p.lfo1_depth), OLED_TEAL, 1);
      for (let i = 0; i < 8; i++) drawParamBar(ctx, 2 + i * 16, 40, 14, 3, p[paramNames[i]] ?? 0, OLED_TEAL);
      break;
    }
    case "trig": {
      const pg = track.pages[0];
      for (let i = 0; i < 16; i++) {
        const step = pg.steps[i];
        const color = step.trigType === "none" ? OLED_DIM : step.trigType === "lock" ? OLED_ORANGE : OLED_TEAL;
        ctx.fillStyle = color;
        ctx.fillRect((2 + i * 8) * 4, 12 * 4, 6 * 4, 6 * 4);
      }
      for (let i = 0; i < 8; i++) drawParamBar(ctx, 2 + i * 16, 45, 14, 3, 0, OLED_TEAL);
      break;
    }
  }
}
