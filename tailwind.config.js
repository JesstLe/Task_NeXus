/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        background: '#f5f7fa',
        primary: '#007aff',
        'primary-hover': '#0062cc',
        surface: '#ffffff',
        text: '#333333',
        'text-secondary': '#666666',
        success: '#10b981',
        error: '#ef4444',
        border: '#e5e7eb'
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
      }
    },
  },
  plugins: [],
}
