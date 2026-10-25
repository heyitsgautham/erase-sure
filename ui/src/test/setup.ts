import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach, beforeEach } from 'vitest';
import { installFakeTauri } from './fakeTauri';

// Every test starts with a fresh fake backend, so screens that subscribe on mount never reach for
// a real one. A test that needs canned answers installs its own, which replaces this one.
// The fake stays installed after a test, because unsubscribing finishes asynchronously.
beforeEach(() => {
    installFakeTauri();
});

afterEach(() => {
    cleanup();
});
