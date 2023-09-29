/** @type {import('tailwindcss').Config} */
export default {
  darkMode: "class",
  content: ["./src/**/*.{html,tsx}"],
  theme: {
    extend: {
      transitionTimingFunction: {
        bouncy: "cubic-bezier(0.220, 0.440, 0.090, 1.110)",
      },
    },
  },
  plugins: [],
};
