import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach } from 'vitest';
import { uninstallFakeTauri } from './fakeTauri';

afterEach(() => {
    cleanup();
    uninstallFakeTauri();
});
