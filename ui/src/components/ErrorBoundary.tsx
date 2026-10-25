import React from 'react';
import { createLogger } from '../lib/logger';

const log = createLogger('error-boundary');

interface ErrorBoundaryState {
    hasError: boolean;
    error?: Error;
}

interface ErrorBoundaryProps {
    children: React.ReactNode;
    /** What failed, in the user's terms. */
    what?: string;
}

/**
 * Catch a render error below it, log it, and show what happened with a way
 * to try again, instead of leaving a blank window.
 */
class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
    constructor(props: ErrorBoundaryProps) {
        super(props);
        this.state = { hasError: false };
    }

    static getDerivedStateFromError(error: Error): ErrorBoundaryState {
        return { hasError: true, error };
    }

    componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
        log.error('render failed', { what: this.props.what, componentStack: errorInfo.componentStack }, error);
    }

    private reset = () => {
        this.setState({ hasError: false, error: undefined });
    };

    render() {
        if (this.state.hasError) {
            return (
                <div className="card" style={{ padding: '2rem', margin: '2rem' }}>
                    <h2 style={{ color: '#ef4444', marginBottom: '1rem' }}>Something went wrong</h2>
                    <p style={{ marginBottom: '1rem' }}>{this.props.what ?? 'This screen'} encountered an error:</p>
                    <pre
                        style={{
                            background: '#f3f4f6',
                            padding: '1rem',
                            borderRadius: '0.5rem',
                            fontSize: '0.875rem',
                            overflow: 'auto',
                        }}
                    >
                        {this.state.error?.toString()}
                    </pre>
                    {!import.meta.env.PROD && (
                        <pre
                            style={{
                                background: '#f3f4f6',
                                padding: '1rem',
                                borderRadius: '0.5rem',
                                fontSize: '0.875rem',
                                overflow: 'auto',
                                marginTop: '0.5rem',
                            }}
                        >
                            {this.state.error?.stack}
                        </pre>
                    )}
                    <button className="btn btn-secondary" style={{ marginTop: '1rem' }} onClick={this.reset}>
                        Try again
                    </button>
                </div>
            );
        }

        return this.props.children;
    }
}

export default ErrorBoundary;
