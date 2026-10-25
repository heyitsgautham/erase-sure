/** Render helpers that wrap a component in the providers the app gives it. */
import { render } from '@testing-library/react';
import type { ReactElement } from 'react';
import { MemoryRouter, Route, Routes, useLocation } from 'react-router-dom';
import { AppProvider } from '../contexts/AppContext';
import type { AppState } from '../contexts/AppContext';

/** Shows the current path so tests can assert where a screen navigated to. */
function LocationProbe() {
    const location = useLocation();
    return <div data-testid="location">{location.pathname}</div>;
}

export function renderWithApp(
    ui: ReactElement,
    { route = '/', state }: { route?: string; state?: Partial<AppState> } = {},
) {
    return render(
        <MemoryRouter initialEntries={[route]} future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>
            <AppProvider initial={state}>
                <Routes>
                    <Route path="*" element={ui} />
                </Routes>
                <LocationProbe />
            </AppProvider>
        </MemoryRouter>,
    );
}
