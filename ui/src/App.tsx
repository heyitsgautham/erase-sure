import { Routes, Route, useLocation } from 'react-router-dom';
import { AppProvider } from './contexts/AppContext';
import Navigation from './components/Navigation';
import Home from './screens/Home';
import Discover from './screens/Discover';
import BackupDiscover from './screens/BackupDiscover';
import WipePlanDiscover from './screens/WipePlanDiscover';
import DestructiveWipeDiscover from './screens/DestructiveWipeDiscover';
import WipePlan from './screens/WipePlan';
import DestructiveWipe from './screens/DestructiveWipe';
import Backup from './screens/Backup';
import Certificates from './screens/Certificates';
import FileBrowserDemo from './screens/FileBrowserDemo';
import Toast from './components/Toast';
import CliLogStream from './components/CliLogStream';
import ErrorBoundary from './components/ErrorBoundary';

function App() {
    // Keyed by path, so moving to another screen clears an error shown on the last one.
    const location = useLocation();

    return (
        <AppProvider>
            <CliLogStream />
            <div className="app">
                <Navigation />
                <main className="main-content">
                    <ErrorBoundary key={location.pathname}>
                        <Routes>
                            <Route path="/" element={<Home />} />
                            <Route path="/discover" element={<Discover />} />
                            <Route path="/backup-discover" element={<BackupDiscover />} />
                            <Route path="/wipe-plan-discover" element={<WipePlanDiscover />} />
                            <Route path="/destructive-wipe-discover" element={<DestructiveWipeDiscover />} />
                            <Route path="/wipe-plan" element={<WipePlan />} />
                            <Route path="/destructive-wipe" element={<DestructiveWipe />} />
                            <Route path="/backup" element={<Backup />} />
                            <Route path="/certificates" element={<Certificates />} />
                            <Route path="/file-browser-demo" element={<FileBrowserDemo />} />
                        </Routes>
                    </ErrorBoundary>
                </main>
                <Toast />
            </div>
        </AppProvider>
    );
}

export default App;
