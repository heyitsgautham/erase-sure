interface NavigationProps {
    title: string;
}

function Navigation({ title }: NavigationProps) {
    return (
        <nav className="nav">
            <div>
                <h1>SecureWipe</h1>
            </div>
            <div className="nav-breadcrumbs">
                <span>{title}</span>
            </div>
        </nav>
    );
}

export default Navigation;
