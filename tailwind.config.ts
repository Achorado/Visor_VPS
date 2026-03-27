import type { Config } from 'tailwindcss';

export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	darkMode: 'class',
	theme: {
		extend: {
			colors: {
				primary: {
					50: '#eef2ff',
					100: '#e0e7ff',
					400: '#818cf8',
					500: '#6366f1',
					600: '#4f46e5',
					700: '#4338ca',
					900: '#312e81'
				},
				surface: {
					50: '#f8fafc',
					100: '#f1f5f9',
					800: '#1e293b',
					850: '#172033',
					900: '#0f172a',
					950: '#080d1a'
				},
				success: '#22c55e',
				warning: '#f59e0b',
				danger: '#ef4444',
				info: '#38bdf8'
			},
			fontFamily: {
				sans: ['Inter', 'system-ui', 'sans-serif'],
				mono: ['JetBrains Mono', 'Fira Code', 'monospace']
			},
			animation: {
				'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
				'fade-in': 'fadeIn 0.3s ease-out',
				'slide-in': 'slideIn 0.3s ease-out',
				shimmer: 'shimmer 2s linear infinite'
			},
			keyframes: {
				fadeIn: {
					'0%': { opacity: '0', transform: 'translateY(8px)' },
					'100%': { opacity: '1', transform: 'translateY(0)' }
				},
				slideIn: {
					'0%': { opacity: '0', transform: 'translateX(-12px)' },
					'100%': { opacity: '1', transform: 'translateX(0)' }
				},
				shimmer: {
					'0%': { backgroundPosition: '-200% 0' },
					'100%': { backgroundPosition: '200% 0' }
				}
			},
			backgroundImage: {
				'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
				'noise':
					"url(\"data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)' opacity='0.4'/%3E%3C/svg%3E\")"
			}
		}
	},
	plugins: []
} satisfies Config;
