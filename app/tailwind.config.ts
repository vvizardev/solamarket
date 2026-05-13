import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./src/**/*.{js,ts,jsx,tsx,mdx}"],
  theme: {
    extend: {
      colors: {
        yes:  { DEFAULT: "#22c55e", muted: "#16a34a" },
        no:   { DEFAULT: "#ef4444", muted: "#b91c1c" },
        surface: "#0f172a",
        panel:   "#1e293b",
        border:  "#334155",
      },
      fontFamily: {
        mono: ["'JetBrains Mono'", "monospace"],
      },
    },
  },
  plugins: [],
};

export default config;
