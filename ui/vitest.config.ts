import { defineConfig } from 'vitest/config';

export default defineConfig({
    test: {
        include: ['src/**/*.test.{ts,tsx}'],
        environment: 'jsdom',
        setupFiles: ['src/test/setup.ts'],
        restoreMocks: true,
        coverage: {
            provider: 'v8',
            include: ['src/**/*.{ts,tsx}'],
            exclude: ['src/**/*.test.{ts,tsx}', 'src/test/**', 'src/main.tsx'],
            reporter: ['text-summary', 'text'],
            thresholds: { lines: 90, statements: 90, functions: 85, branches: 85 },
        },
    },
});
