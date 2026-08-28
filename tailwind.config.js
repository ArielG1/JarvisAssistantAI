/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        jarvis: {
          bg: '#0a0a0f',
          panel: '#12121a',
          border: '#1e1e2e',
          text: '#e0e0e0',
          muted: '#6b7280',
          cyan: '#00d4ff',
          violet: '#8b5cf6',
          amber: '#f59e0b',
          green: '#10b981',
          red: '#ef4444',
        }
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'Fira Code', 'monospace'],
      }
    },
  },
  plugins: [],
};