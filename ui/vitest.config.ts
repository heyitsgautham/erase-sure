import { defineConfig } from "vitest/config";

export default defineConfig({
    test: {
        include: ["src/**/*.test.ts"],
        environment: "node",
        coverage: {
            provider: "v8",
            // Screens need the Tauri runtime; unit tests cover the logic pulled out into src/lib.
            include: ["src/lib/**/*.ts"],
            exclude: ["src/**/*.test.ts"],
            reporter: ["text"],
            thresholds: { lines: 90, branches: 85, functions: 90, statements: 90 },
        },
    },
});
