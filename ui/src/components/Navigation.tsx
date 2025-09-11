import { useNavigate, useLocation } from 'react-router-dom';

interface NavigationProps {
    title?: string; // Optional since we now use breadcrumbs
}

function Navigation({ }: NavigationProps) {
    const navigate = useNavigate();
    const location = useLocation();

    const getBreadcrumbs = () => {
        const path = location.pathname;
        const breadcrumbs = [];

        breadcrumbs.push({ label: 'Home', path: '/', current: path === '/' });

        if (path === '/discover') breadcrumbs.push({ label: 'Device Discovery', path: '/discover', current: true });
        if (path === '/wipe-plan') breadcrumbs.push({ label: 'Wipe Plan', path: '/wipe-plan', current: true });
        if (path === '/backup') breadcrumbs.push({ label: 'Backup', path: '/backup', current: true });
        if (path === '/certificates') breadcrumbs.push({ label: 'Certificates', path: '/certificates', current: true });

        return breadcrumbs;
    };

    const breadcrumbs = getBreadcrumbs();

    return (
        <nav className="nav">
            <div>
                <h1
                    onClick={() => navigate('/')}
                    style={{ cursor: 'pointer' }}
                    title="Go to Home"
                >
                    SecureWipe
                </h1>
            </div>
            <div className="nav-breadcrumbs">
                {breadcrumbs.map((crumb, index) => (
                    <span key={crumb.path}>
                        {index > 0 && <span className="breadcrumb-separator"> / </span>}
                        <span
                            className={crumb.current ? 'breadcrumb-current' : 'breadcrumb-link'}
                            onClick={() => !crumb.current && navigate(crumb.path)}
                            style={{ cursor: crumb.current ? 'default' : 'pointer' }}
                        >
                            {crumb.label}
                        </span>
                    </span>
                ))}
            </div>
        </nav>
    );
}

export default Navigation;
