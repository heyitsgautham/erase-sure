#!/usr/bin/env python3
"""
Run the SecureWipe Verification Portal

Usage:
    python run.py [--host 0.0.0.0] [--port 8000]
"""

import argparse
import uvicorn


def main():
    parser = argparse.ArgumentParser(description="Run SecureWipe Verification Portal")
    parser.add_argument("--host", default="0.0.0.0", help="Host to bind to")
    parser.add_argument("--port", type=int, default=8000, help="Port to bind to")
    parser.add_argument("--reload", action="store_true", help="Enable auto-reload for development")
    
    args = parser.parse_args()
    
    uvicorn.run(
        "app.main:app",
        host=args.host,
        port=args.port,
        reload=args.reload
    )


if __name__ == "__main__":
    main()
