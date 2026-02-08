/** @type {import('tailwindcss').Config} */
export default {
  content: ["./app/**/{**,.client,.server}/**/*.{js,jsx,ts,tsx}"],
  theme: {
    extend: {
      colors: {
        chassis: {
          950: "#0a0a0c",
          900: "#111114",
          800: "#1a1a1f",
          700: "#252529",
          600: "#303036",
        },
        oled: {
          orange: "#ff6a00",
          teal: "#00e5c8",
          amber: "#ffb300",
          red: "#ff2d2d",
          green: "#00ff6a",
          dim: "#333333",
        },
        track: {
          audio: "#22c55e",
          bus: "#3b82f6",
          send: "#a855f7",
          mix: "#eab308",
        },
      },
      fontFamily: {
        mono: ['"JetBrains Mono"', '"Fira Code"', "monospace"],
        display: ['"Inter"', "system-ui", "sans-serif"],
      },
    },
  },
  plugins: [],
};
