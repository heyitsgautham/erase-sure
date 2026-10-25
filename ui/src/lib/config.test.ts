import { describe, expect, it } from 'vitest';
import { DEFAULT_PORTAL_URL, resolvePortalUrl } from './config';

describe('resolvePortalUrl', () => {
    it('uses the local portal when nothing is set', () => {
        expect(resolvePortalUrl(undefined)).toBe(DEFAULT_PORTAL_URL);
        expect(resolvePortalUrl('  ')).toBe(DEFAULT_PORTAL_URL);
    });

    it('accepts http and https URLs and drops a trailing slash', () => {
        expect(resolvePortalUrl('https://verify.example.org/')).toBe('https://verify.example.org');
        expect(resolvePortalUrl('http://10.0.0.5:9000/portal/')).toBe('http://10.0.0.5:9000/portal');
    });

    it('ignores values that are not web URLs', () => {
        expect(resolvePortalUrl('javascript:alert(1)')).toBe(DEFAULT_PORTAL_URL);
        expect(resolvePortalUrl('file:///etc/passwd')).toBe(DEFAULT_PORTAL_URL);
        expect(resolvePortalUrl('not a url')).toBe(DEFAULT_PORTAL_URL);
    });
});
