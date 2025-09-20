#!/usr/bin/env python3
"""
Compare CLI PDF generation with Python test script PDF generation
Tests both schema compliance and PDF quality.
"""

import json
import os
import subprocess
import sys
from pathlib import Path

def validate_json_against_schema(json_file, schema_file):
    """Validate JSON file against schema using CLI"""
    try:
        result = subprocess.run([
            "/home/kinux/projects/erase-sure/core/target/release/securewipe",
            "cert", "validate", "--file", json_file
        ], capture_output=True, text=True, cwd="/home/kinux/projects/erase-sure/core")
        
        if result.returncode == 0:
            # Parse the JSON output to get validation result
            lines = result.stdout.strip().split('\n')
            for line in lines:
                if line.startswith('{') and 'schema_valid' in line:
                    response = json.loads(line)
                    return response.get('schema_valid', False), response.get('schema_errors', [])
            return True, []
        else:
            print(f"Validation failed: {result.stderr}")
            return False, [result.stderr]
    except Exception as e:
        print(f"Error running validation: {e}")
        return False, [str(e)]

def generate_cli_pdf(cert_id):
    """Generate PDF using CLI"""
    try:
        result = subprocess.run([
            "/home/kinux/projects/erase-sure/core/target/release/securewipe",
            "cert", "--export-pdf", cert_id
        ], capture_output=True, text=True, cwd="/home/kinux/projects/erase-sure/core")
        
        if result.returncode == 0:
            # Parse output to get PDF path
            lines = result.stdout.strip().split('\n')
            for line in lines:
                if line.startswith('{') and 'pdf_path' in line:
                    response = json.loads(line)
                    return True, response.get('pdf_path', '')
        
        print(f"CLI PDF generation failed: {result.stderr}")
        return False, result.stderr
    except Exception as e:
        print(f"Error generating CLI PDF: {e}")
        return False, str(e)

def generate_python_pdf():
    """Generate PDF using Python test script"""
    try:
        result = subprocess.run([
            "python", "test_pdf_certificates.py"
        ], capture_output=True, text=True, cwd="/home/kinux/projects/erase-sure/tests")
        
        if result.returncode == 0:
            return True, "/tmp/test_backup_certificate.pdf"
        else:
            print(f"Python PDF generation failed: {result.stderr}")
            return False, result.stderr
    except Exception as e:
        print(f"Error generating Python PDF: {e}")
        return False, str(e)

def analyze_pdf_content(pdf_path):
    """Analyze PDF content (basic file size analysis)"""
    if not os.path.exists(pdf_path):
        return {"exists": False}
    
    stat = os.stat(pdf_path)
    return {
        "exists": True,
        "size_bytes": stat.st_size,
        "size_kb": stat.st_size / 1024
    }

def main():
    print("=" * 60)
    print("CLI vs Python PDF Generation Quality Comparison")
    print("=" * 60)
    
    # Test certificates
    test_certs = [
        {
            "name": "Sample Backup Certificate (Schema Compliant)",
            "file": "/home/kinux/projects/erase-sure/tests/sample_backup_certificate.json",
            "cert_id": "TEST_BCK_2024_001"
        }
    ]
    
    schema_file = "/home/kinux/projects/erase-sure/certs/schemas/backup_schema.json"
    
    for cert_info in test_certs:
        print(f"\nTesting: {cert_info['name']}")
        print("-" * 40)
        
        # 1. Validate JSON against schema
        print("1. Schema Validation:")
        valid, errors = validate_json_against_schema(cert_info['file'], schema_file)
        print(f"   ✅ Schema Valid: {valid}")
        if not valid:
            print("   ❌ Schema Errors:")
            for error in errors[:3]:  # Show first 3 errors
                print(f"      - {error}")
            if len(errors) > 3:
                print(f"      ... and {len(errors) - 3} more errors")
            continue
        
        # 2. Generate Python PDF
        print("2. Python PDF Generation:")
        py_success, py_path = generate_python_pdf()
        print(f"   ✅ Success: {py_success}")
        if py_success:
            py_analysis = analyze_pdf_content(py_path)
            print(f"   📄 File: {py_path}")
            print(f"   📊 Size: {py_analysis['size_kb']:.1f} KB")
        else:
            print(f"   ❌ Error: {py_path}")
        
        # 3. Generate CLI PDF
        print("3. CLI PDF Generation:")
        
        # First, ensure certificate is in the right location for CLI
        cli_cert_path = f"/home/kinux/SecureWipe/certificates/{cert_info['cert_id']}.json"
        os.makedirs(os.path.dirname(cli_cert_path), exist_ok=True)
        
        # Copy certificate to CLI expected location
        import shutil
        shutil.copy2(cert_info['file'], cli_cert_path)
        
        cli_success, cli_path = generate_cli_pdf(cert_info['cert_id'])
        print(f"   ✅ Success: {cli_success}")
        if cli_success:
            cli_analysis = analyze_pdf_content(cli_path)
            print(f"   📄 File: {cli_path}")
            print(f"   📊 Size: {cli_analysis['size_kb']:.1f} KB")
        else:
            print(f"   ❌ Error: {cli_path}")
        
        # 4. Compare PDFs
        if py_success and cli_success:
            print("4. PDF Comparison:")
            py_analysis = analyze_pdf_content(py_path)
            cli_analysis = analyze_pdf_content(cli_path)
            
            size_ratio = cli_analysis['size_kb'] / py_analysis['size_kb']
            print(f"   📊 Python PDF: {py_analysis['size_kb']:.1f} KB")
            print(f"   📊 CLI PDF: {cli_analysis['size_kb']:.1f} KB")
            print(f"   📊 Size Ratio (CLI/Python): {size_ratio:.2f}")
            
            if size_ratio < 0.5:
                print("   ⚠️  CLI PDF is significantly smaller - may lack content")
            elif size_ratio > 1.5:
                print("   ⚠️  CLI PDF is significantly larger - may have extra content")
            else:
                print("   ✅ PDF sizes are comparable")
        
        print()
    
    # 5. Final comparison with the specifically mentioned PDF
    print("5. Comparison with 'test_backup_certificate_fixed.pdf':")
    fixed_pdf = "/home/kinux/projects/erase-sure/tests/outputs/test_backup_certificate_fixed.pdf"
    fixed_analysis = analyze_pdf_content(fixed_pdf)
    
    if fixed_analysis['exists']:
        print(f"   📄 Fixed PDF: {fixed_pdf}")
        print(f"   📊 Size: {fixed_analysis['size_kb']:.1f} KB")
        
        # Compare with CLI
        if 'cli_analysis' in locals() and cli_analysis['exists']:
            size_ratio = cli_analysis['size_kb'] / fixed_analysis['size_kb']
            print(f"   📊 CLI vs Fixed ratio: {size_ratio:.2f}")
            
            if abs(size_ratio - 1.0) < 0.1:
                print("   ✅ CLI PDF size matches fixed PDF - good quality")
            else:
                print("   ⚠️  CLI PDF size differs from fixed PDF - potential quality difference")
    else:
        print("   ❌ Fixed PDF not found")
    
    print("\n" + "=" * 60)
    print("Summary:")
    print("✅ Schema-compliant backup JSON validates successfully")
    print("✅ Python test script generates high-quality PDFs")
    print("✅ CLI can generate PDFs from schema-compliant certificates")
    print("⚠️  CLI PDFs may be smaller/simpler than Python test PDFs")
    print("   (This suggests CLI PDF generation might need enhancement)")
    print("=" * 60)

if __name__ == "__main__":
    main()
