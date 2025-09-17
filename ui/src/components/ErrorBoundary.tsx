import React from 'react';

interface ErrorBoundaryState {
  hasError: boolean;
  error?: Error;
}

interface ErrorBoundaryProps {
  children: React.ReactNode;
}

class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('ErrorBoundary caught an error:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="card" style={{ padding: '2rem', margin: '2rem' }}>
          <h2 style={{ color: '#ef4444', marginBottom: '1rem' }}>Something went wrong</h2>
          <p style={{ marginBottom: '1rem' }}>
            The file browser component encountered an error:
          </p>
          <pre style={{ 
            background: '#f3f4f6', 
            padding: '1rem', 
            borderRadius: '0.5rem',
            fontSize: '0.875rem',
            overflow: 'auto'
          }}>
            {this.state.error?.toString()}
          </pre>
          <pre style={{ 
            background: '#f3f4f6', 
            padding: '1rem', 
            borderRadius: '0.5rem',
            fontSize: '0.875rem',
            overflow: 'auto',
            marginTop: '0.5rem'
          }}>
            {this.state.error?.stack}
          </pre>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
