/** Every screen's path and the title the header shows for it. */
export const ROUTE_TITLES: Record<string, string> = {
    '/': 'Home',
    '/discover': 'Device Discovery',
    '/backup-discover': 'Backup Device Discovery',
    '/wipe-plan-discover': 'Wipe Plan Device Discovery',
    '/destructive-wipe-discover': 'Destructive Wipe Device Discovery',
    '/wipe-plan': 'Wipe Plan',
    '/destructive-wipe': 'Destructive Wipe',
    '/backup': 'Backup',
    '/certificates': 'Certificates',
    '/file-browser-demo': 'File Browser Demo',
};

export interface Breadcrumb {
    label: string;
    path: string;
    current: boolean;
}

/** Home, then the current screen when it is not home. Unknown paths show Home only. */
export function breadcrumbsFor(pathname: string): Breadcrumb[] {
    const crumbs: Breadcrumb[] = [{ label: 'Home', path: '/', current: pathname === '/' }];
    const title = ROUTE_TITLES[pathname];
    if (pathname !== '/' && title) {
        crumbs.push({ label: title, path: pathname, current: true });
    }
    return crumbs;
}
