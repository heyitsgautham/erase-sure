import { useNavigate, useLocation } from 'react-router-dom';
import { breadcrumbsFor } from '../lib/routes';

function Navigation() {
    const navigate = useNavigate();
    const location = useLocation();
    const breadcrumbs = breadcrumbsFor(location.pathname);

    return (
        <nav className="nav">
            <div>
                <h1 onClick={() => navigate('/')} style={{ cursor: 'pointer' }} title="Go to Home">
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
