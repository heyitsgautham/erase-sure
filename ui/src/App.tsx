import { Routes, Route, useLocation } from "react-router-dom";
import { AppProvider } from "./contexts/AppContext";
import Navigation from "./components/Navigation";
import Home from "./screens/Home";
import Discover from "./screens/Discover";
import WipePlan from "./screens/WipePlan";
import Backup from "./screens/Backup";
import Certificates from "./screens/Certificates";
import FileBrowserDemo from "./screens/FileBrowserDemo";
import Toast from "./components/Toast";

function App() {
    const location = useLocation();

    const getPageTitle = () => {
        switch (location.pathname) {
            case "/": return "Home";
            case "/discover": return "Device Discovery";
            case "/wipe-plan": return "Wipe Plan";
            case "/backup": return "Backup";
            case "/certificates": return "Certificates";
            case "/file-browser-demo": return "File Browser Demo";
            default: return "SecureWipe";
        }
    };

    return (
        <AppProvider>
            <div className="app">
                <Navigation title={getPageTitle()} />
                <main className="main-content">
                    <Routes>
                        <Route path="/" element={<Home />} />
                        <Route path="/discover" element={<Discover />} />
                        <Route path="/wipe-plan" element={<WipePlan />} />
                        <Route path="/backup" element={<Backup />} />
                        <Route path="/certificates" element={<Certificates />} />
                        <Route path="/file-browser-demo" element={<FileBrowserDemo />} />
                    </Routes>
                </main>
                <Toast />
            </div>
        </AppProvider>
    );
}

export default App;
