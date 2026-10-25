/**
 * Settings read from the build environment.
 *
 * Vite inlines `VITE_*` variables at build time; see ui/.env.example.
 */

export const DEFAULT_PORTAL_URL = 'http://localhost:8000';

/**
 * The verification portal's base URL, without a trailing slash.
 * Anything that is not an http or https URL falls back to the local portal.
 */
export function resolvePortalUrl(raw: string | undefined): string {
    const value = raw?.trim();
    if (!value) return DEFAULT_PORTAL_URL;
    try {
        const url = new URL(value);
        if (url.protocol !== 'http:' && url.protocol !== 'https:') return DEFAULT_PORTAL_URL;
        return url.toString().replace(/\/+$/, '');
    } catch {
        return DEFAULT_PORTAL_URL;
    }
}

export const PORTAL_URL = resolvePortalUrl(import.meta.env.VITE_PORTAL_URL);
