#!/usr/bin/env python3
"""
High-quality PDF generator for SecureWipe certificates
Called by Rust CLI to generate professional PDFs using reportlab
"""

import argparse
import json
import sys
import os
import tempfile
from pathlib import Path
from datetime import datetime, timezone

# Add the tests directory to Python path to import existing generators
script_dir = Path(__file__).parent
project_root = script_dir.parent.parent  # Go up to project root
tests_dir = project_root / "tests"
sys.path.insert(0, str(tests_dir))

try:
    import jsonschema
    from jsonschema import validate, ValidationError
except ImportError:
    print("❌ Missing dependency: jsonschema")
    print("   Install with: pip install jsonschema")
    sys.exit(1)

try:
    from reportlab.lib.pagesizes import letter, A4
    from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
    from reportlab.lib.units import inch
    from reportlab.lib import colors
    from reportlab.platypus import SimpleDocTemplate, Table, TableStyle, Paragraph, Spacer, Image
    from reportlab.lib.enums import TA_LEFT, TA_CENTER, TA_RIGHT
    from reportlab.graphics.shapes import Drawing, Rect
    from reportlab.graphics import renderPDF
except ImportError:
    print("❌ Missing dependency: reportlab")
    print("   Install with: pip install reportlab")
    sys.exit(1)

try:
    import qrcode
    from PIL import Image as PILImage
except ImportError:
    print("❌ Missing dependency: qrcode[pil]")
    print("   Install with: pip install qrcode[pil]")
    sys.exit(1)

# Import existing PDF generation functions
try:
    from test_pdf_certificates import BackupCertificatePDFGenerator
    from test_wipe_pdf_certificates import create_wipe_certificate_pdf, create_sample_wipe_certificate
except ImportError as e:
    print(f"❌ Could not import existing PDF generators: {e}")
    print("   Make sure you're running from the project root")
    sys.exit(1)


def load_schema(schema_type):
    """Load the appropriate JSON schema"""
    schema_path = project_root / "certs" / "schemas" / f"{schema_type}_schema.json"
    if not schema_path.exists():
        raise FileNotFoundError(f"Schema not found: {schema_path}")
    
    with open(schema_path, 'r') as f:
        return json.load(f)


def validate_certificate(cert_data, schema_type):
    """Validate certificate against schema"""
    try:
        schema = load_schema(schema_type)
        validate(instance=cert_data, schema=schema)
        return True, []
    except ValidationError as e:
        return False, [str(e)]
    except Exception as e:
        return False, [f"Validation error: {e}"]


def generate_backup_pdf(cert_data, output_path, skip_validation=False):
    """Generate backup certificate PDF using existing high-quality generator"""
    try:
        # Use the existing BackupCertificatePDFGenerator
        generator = BackupCertificatePDFGenerator()
        
        # Only validate if explicitly requested and not skipped
        if not skip_validation:
            is_valid = generator.validate_certificate(cert_data)
            if not is_valid:
                print("⚠️  Certificate validation failed, but proceeding with PDF generation")
                # Don't raise error - proceed with PDF generation for unsigned certs
        
        # Generate PDF
        success = generator.create_certificate_pdf(cert_data, output_path, skip_validation)
        if not success:
            raise RuntimeError("PDF generation failed")
        
        return True, output_path
    except Exception as e:
        return False, str(e)


def generate_wipe_pdf(cert_data, output_path):
    """Generate wipe certificate PDF using existing high-quality generator"""
    try:
        # Use the existing wipe certificate generator
        create_wipe_certificate_pdf(cert_data, output_path)
        return True, output_path
    except Exception as e:
        return False, str(e)


def main():
    parser = argparse.ArgumentParser(description='Generate high-quality PDF certificates')
    parser.add_argument('--cert-file', required=True, help='Path to certificate JSON file')
    parser.add_argument('--output', required=True, help='Output PDF path')
    parser.add_argument('--type', choices=['backup', 'wipe'], required=True, 
                       help='Certificate type')
    parser.add_argument('--validate', action='store_true', default=False,
                       help='Validate certificate against schema')
    parser.add_argument('--no-validate', action='store_true',
                        help='Skip certificate schema validation (default)')
    
    args = parser.parse_args()
    
    # Load certificate
    try:
        with open(args.cert_file, 'r') as f:
            cert_data = json.load(f)
    except Exception as e:
        print(f"❌ Failed to load certificate: {e}", file=sys.stderr)
        sys.exit(1)
    
    # Validate only if explicitly requested
    should_validate = args.validate and not args.no_validate
    if should_validate:
        is_valid, errors = validate_certificate(cert_data, args.type)
        if not is_valid:
            print(f"❌ Certificate validation failed:", file=sys.stderr)
            for error in errors[:3]:  # Show first 3 errors
                print(f"   - {error}", file=sys.stderr)
            sys.exit(1)
        print("✅ Certificate validation passed")
    else:
        print("⚠️  Schema validation skipped")
    
    
    # Ensure output directory exists
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    # Generate PDF based on type
    print(f"📄 Generating {args.type} certificate PDF...")
    
    skip_validation = args.no_validate
    if args.type == 'backup':
        success, result = generate_backup_pdf(cert_data, str(output_path), skip_validation)
    else:  # wipe
        success, result = generate_wipe_pdf(cert_data, str(output_path))
    
    if success:
        file_size = os.path.getsize(output_path)
        print(f"✅ PDF generated successfully: {output_path}")
        print(f"📊 File size: {file_size:,} bytes ({file_size/1024:.1f} KB)")
        
        # Output structured result for CLI parsing
        result_data = {
            "success": True,
            "output_path": str(output_path),
            "file_size_bytes": file_size,
            "certificate_type": args.type,
            "timestamp": datetime.now(timezone.utc).isoformat()
        }
        print(f"RESULT_JSON: {json.dumps(result_data)}")
    else:
        print(f"❌ PDF generation failed: {result}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
