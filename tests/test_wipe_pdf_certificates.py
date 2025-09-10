#!/usr/bin/env python3
"""
SecureWipe Wipe Certificate PDF Generation Test

Tests PDF generation for audit-ready wipe certificates with:
- Professional layout and styling
- QR code integration
- Comprehensive data tables
- Security validation
"""

import json
import sys
import os
import tempfile
from datetime import datetime, timezone
from pathlib import Path

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

# Get project root and schema path
PROJECT_ROOT = Path(__file__).parent.parent
SCHEMA_PATH = PROJECT_ROOT / "certs" / "schemas" / "wipe_schema.json"

def load_schema():
    """Load the wipe certificate JSON schema"""
    with open(SCHEMA_PATH, 'r') as f:
        return json.load(f)

def create_sample_wipe_certificate():
    """Create a sample wipe certificate for PDF generation"""
    return {
        "cert_type": "wipe",
        "cert_id": "TEST_WPE_2024_001",
        "certificate_version": "v1.0.0", 
        "created_at": "2024-01-15T11:45:30+05:30",
        "issuer": {
            "organization": "SecureWipe (SIH)",
            "tool_name": "securewipe",
            "tool_version": "v2.1.0",
            "country": "IN"
        },
        "device": {
            "model": "Samsung SSD 980 PRO 1TB",
            "serial": "S6TXNX0R123456",
            "bus": "NVMe", 
            "capacity_bytes": 1000204886016,
            "path": "/dev/nvme0n1",
            "firmware": "5B2QGXA7",
            "logical_block_size": 512,
            "total_lbas": 1953525168,
            "protocol_path": "PCIe->NVMe"
        },
        "policy": {
            "nist_level": "PURGE",
            "method": "nvme_sanitize_crypto_erase",
            "action_mapping": "Sanitize → Crypto Erase → PURGE"
        },
        "hpa_dco": {
            "cleared": True,
            "commands": [
                "nvme sanitize /dev/nvme0n1 --crypto-erase",
                "nvme sanitize-log /dev/nvme0n1"
            ]
        },
        "commands": [
            {
                "cmd": "nvme sanitize /dev/nvme0n1 --crypto-erase",
                "exit": 0,
                "ms": 45780
            },
            {
                "cmd": "nvme sanitize-log /dev/nvme0n1", 
                "exit": 0,
                "ms": 120
            }
        ],
        "verify": {
            "strategy": "controller_status",
            "coverage": {
                "mode": "samples",
                "samples": 50
            },
            "failures": 0,
            "result": "PASS"
        },
        "result": "PASS",
        "environment": {
            "operator": "admin",
            "os_kernel": "Linux 6.8.0-35-generic",
            "tool_version": "v2.1.0",
            "device_firmware": "5B2QGXA7"
        },
        "evidence": {
            "nvme_sanitize_status_code": "0x0000",
            "logs_sha256": "c3d4e5f6789012345678901234567890123456789012345678901234567890ab"
        },
        "linkage": {
            "backup_cert_id": "TEST_BCK_2024_001"
        },
        "exceptions": {
            "text": "None"
        },
        "signature": {
            "alg": "Ed25519",
            "pubkey_id": "sih_root_v1",
            "sig": "xyzabc7890123456789012345678901234567890123456789012345678901234567890xyz=="
        },
        "metadata": {
            "certificate_json_sha256": "f6789012345678901234567890123456789012345678901234567890abcdef12",
            "qr_payload": {
                "cert_id": "TEST_WPE_2024_001",
                "issued_at": "2024-01-15T11:45:30+05:30",
                "device_model": "Samsung SSD 980 PRO 1TB",
                "result": "PASS",
                "nist_level": "PURGE",
                "method": "nvme_sanitize_crypto_erase",
                "verify_url": "https://verify.securewipe.org/cert/TEST_WPE_2024_001"
            }
        },
        "verify_url": "https://verify.securewipe.org/cert/TEST_WPE_2024_001"
    }

def generate_qr_code(data):
    """Generate QR code for certificate verification"""
    qr = qrcode.QRCode(
        version=1,
        error_correction=qrcode.constants.ERROR_CORRECT_L,
        box_size=10,
        border=4,
    )
    
    if isinstance(data, dict):
        # Use the verify_url directly if available, otherwise use cert_id
        qr_text = data.get('verify_url', f"cert_id:{data.get('cert_id', 'N/A')}")
    else:
        qr_text = str(data)
    
    qr.add_data(qr_text)
    qr.make(fit=True)
    
    img = qr.make_image(fill_color="black", back_color="white")
    
    # Save to temporary file
    temp_path = tempfile.mktemp(suffix=".png")
    img.save(temp_path)
    return temp_path

def format_bytes(bytes_value):
    """Format bytes into human readable format"""
    if bytes_value >= 1024**4:
        return f"{bytes_value / 1024**4:.2f} TB"
    elif bytes_value >= 1024**3:
        return f"{bytes_value / 1024**3:.2f} GB" 
    elif bytes_value >= 1024**2:
        return f"{bytes_value / 1024**2:.2f} MB"
    elif bytes_value >= 1024:
        return f"{bytes_value / 1024:.2f} KB"
    else:
        return f"{bytes_value} bytes"

def wrap_long_text(text, max_length=50):
    """Wrap long text to fit in table cells"""
    if len(str(text)) <= max_length:
        return str(text)
    return str(text)[:max_length-3] + "..."

def format_hash(hash_value, max_length=32):
    """Format hash values for display in tables"""
    if not hash_value:
        return "N/A"
    hash_str = str(hash_value)
    if len(hash_str) <= max_length:
        return hash_str
    # Show first 16 and last 8 characters with ellipsis
    return f"{hash_str[:16]}...{hash_str[-8:]}"

def format_url(url, max_length=45):
    """Format URLs for display in tables - display only, not for links"""
    if not url:
        return "N/A"
    url_str = str(url)
    if len(url_str) <= max_length:
        return url_str
    # Show protocol and domain, then truncate
    if "://" in url_str:
        protocol_domain = url_str.split("/")[0] + "//" + url_str.split("/")[2]
        if len(protocol_domain) < max_length - 5:
            return protocol_domain + "/..."
    return url_str[:max_length-3] + "..."

def create_clickable_url(url, display_text=None, style=None):
    """Create a clickable URL paragraph with full link preserved"""
    if not url:
        return "N/A"
    
    if display_text is None:
        display_text = format_url(url, 40)
    
    # Create a paragraph with link - style will be provided by caller
    link_text = f'<link href="{url}">{display_text}</link>'
    return Paragraph(link_text, style)

def create_wipe_certificate_pdf(cert_data, output_path):
    """Generate professional PDF certificate for wipe operations"""
    doc = SimpleDocTemplate(output_path, pagesize=A4,
                          rightMargin=72, leftMargin=72,
                          topMargin=72, bottomMargin=18)
    
    styles = getSampleStyleSheet()
    story = []
    
    # Custom styles
    title_style = ParagraphStyle(
        'CustomTitle',
        parent=styles['Heading1'],
        fontSize=24,
        spaceAfter=30,
        alignment=TA_CENTER,
        textColor=colors.darkblue
    )
    
    header_style = ParagraphStyle(
        'CustomHeader',
        parent=styles['Heading2'], 
        fontSize=14,
        spaceAfter=12,
        textColor=colors.darkblue
    )
    
    # Small text style for long content
    small_text_style = ParagraphStyle(
        'SmallText',
        parent=styles['Normal'],
        fontSize=8,
        wordWrap='LTR'
    )
    
    # Title
    story.append(Paragraph("SecureWipe Data Sanitization Certificate", title_style))
    story.append(Paragraph("NIST SP 800-88 Compliant Wipe Operation", styles['Heading3']))
    story.append(Spacer(1, 20))
    
    # Certificate Info Header
    cert_info_data = [
        ["Certificate ID:", cert_data['cert_id']],
        ["Certificate Type:", cert_data['cert_type'].upper()],
        ["Certificate Version:", cert_data['certificate_version']],
        ["Created:", cert_data['created_at']],
        ["Result:", cert_data['result']],
        ["Verification URL:", create_clickable_url(cert_data.get('verify_url', 'N/A'), None, small_text_style)]
    ]
    
    cert_info_table = Table(cert_info_data, colWidths=[2*inch, 3.5*inch])
    cert_info_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (0, -1), colors.lightgrey),
        ('TEXTCOLOR', (0, 0), (-1, -1), colors.black),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('FONTNAME', (0, 0), (-1, -1), 'Helvetica'),
        ('FONTSIZE', (0, 0), (-1, -1), 10),
        ('GRID', (0, 0), (-1, -1), 1, colors.black),
        ('VALIGN', (0, 0), (-1, -1), 'TOP'),
        ('ROWBACKGROUNDS', (0, 0), (-1, -1), [colors.white, colors.lightgrey]),
    ]))
    
    story.append(cert_info_table)
    story.append(Spacer(1, 20))
    
    # Device Information
    story.append(Paragraph("Device Information", header_style))
    device_data = [
        ["Model:", cert_data['device']['model']],
        ["Serial Number:", cert_data['device']['serial']],
        ["Bus Type:", cert_data['device']['bus']],
        ["Capacity:", format_bytes(cert_data['device']['capacity_bytes'])],
        ["Device Path:", cert_data['device'].get('path', 'N/A')],
        ["Protocol Path:", cert_data['device'].get('protocol_path', 'N/A')],
        ["Firmware:", cert_data['device'].get('firmware', 'N/A')]
    ]
    
    device_table = Table(device_data, colWidths=[2*inch, 3.5*inch])
    device_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (0, -1), colors.lightblue),
        ('TEXTCOLOR', (0, 0), (-1, -1), colors.black),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('FONTNAME', (0, 0), (-1, -1), 'Helvetica'),
        ('FONTSIZE', (0, 0), (-1, -1), 9),
        ('GRID', (0, 0), (-1, -1), 1, colors.black),
    ]))
    
    story.append(device_table)
    story.append(Spacer(1, 15))
    
    # Wipe Policy
    story.append(Paragraph("Sanitization Policy", header_style))
    policy_data = [
        ["NIST Level:", cert_data['policy']['nist_level']],
        ["Method:", cert_data['policy']['method']],
        ["Action Mapping:", cert_data['policy'].get('action_mapping', 'N/A')]
    ]
    
    policy_table = Table(policy_data, colWidths=[2*inch, 3.5*inch])
    policy_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (0, -1), colors.lightgreen),
        ('TEXTCOLOR', (0, 0), (-1, -1), colors.black),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'), 
        ('FONTNAME', (0, 0), (-1, -1), 'Helvetica'),
        ('FONTSIZE', (0, 0), (-1, -1), 9),
        ('GRID', (0, 0), (-1, -1), 1, colors.black),
    ]))
    
    story.append(policy_table)
    story.append(Spacer(1, 15))
    
    # HPA/DCO Clearance
    story.append(Paragraph("HPA/DCO Clearance", header_style))
    
    # Format commands properly
    commands_text = ", ".join(cert_data['hpa_dco'].get('commands', []))
    formatted_commands = Paragraph(wrap_long_text(commands_text, 60), small_text_style)
    
    hpa_data = [
        ["Status:", "✓ CLEARED" if cert_data['hpa_dco']['cleared'] else "✗ NOT CLEARED"],
        ["Commands:", formatted_commands]
    ]
    
    hpa_table = Table(hpa_data, colWidths=[2*inch, 3.5*inch])
    hpa_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (0, -1), colors.lightyellow),
        ('TEXTCOLOR', (0, 0), (-1, -1), colors.black),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('FONTNAME', (0, 0), (-1, -1), 'Helvetica'),
        ('FONTSIZE', (0, 0), (-1, -1), 9),
        ('GRID', (0, 0), (-1, -1), 1, colors.black),
    ]))
    
    story.append(hpa_table)
    story.append(Spacer(1, 15))
    
    # Commands Executed
    story.append(Paragraph("Commands Executed", header_style))
    cmd_headers = [["Command", "Exit Code", "Duration (ms)"]]
    cmd_data = []
    for cmd in cert_data['commands']:
        # Use Paragraph for command text to enable proper wrapping
        cmd_text = Paragraph(wrap_long_text(cmd['cmd'], 45), small_text_style)
        cmd_data.append([
            cmd_text,
            str(cmd['exit']),
            str(cmd['ms'])
        ])
    
    cmd_table = Table(cmd_headers + cmd_data, colWidths=[3*inch, 1*inch, 1.5*inch])
    cmd_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), colors.grey),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.whitesmoke),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTNAME', (0, 1), (-1, -1), 'Helvetica'),
        ('FONTSIZE', (0, 0), (-1, -1), 8),
        ('GRID', (0, 0), (-1, -1), 1, colors.black),
        ('VALIGN', (0, 0), (-1, -1), 'TOP'),
    ]))
    
    story.append(cmd_table)
    story.append(Spacer(1, 15))
    
    # Verification Results
    story.append(Paragraph("Verification Results", header_style))
    verify_data = [
        ["Strategy:", cert_data['verify']['strategy']],
        ["Coverage:", f"{cert_data['verify']['coverage']['samples']} samples" if 'coverage' in cert_data['verify'] else 'N/A'],
        ["Failures:", str(cert_data['verify']['failures'])],
        ["Result:", cert_data['verify'].get('result', 'N/A')]
    ]
    
    verify_table = Table(verify_data, colWidths=[2*inch, 3.5*inch])
    verify_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (0, -1), colors.lightcyan),
        ('TEXTCOLOR', (0, 0), (-1, -1), colors.black),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('FONTNAME', (0, 0), (-1, -1), 'Helvetica'),
        ('FONTSIZE', (0, 0), (-1, -1), 9),
        ('GRID', (0, 0), (-1, -1), 1, colors.black),
    ]))
    
    story.append(verify_table)
    story.append(Spacer(1, 15))
    
    # Evidence & Linkage
    story.append(Paragraph("Evidence & Linkage", header_style))
    evidence_data = [
        ["NVMe Status Code:", cert_data['evidence'].get('nvme_sanitize_status_code', 'N/A')],
        ["Logs SHA256:", Paragraph(format_hash(cert_data['evidence'].get('logs_sha256', 'N/A')), small_text_style)],
        ["Backup Certificate:", cert_data['linkage']['backup_cert_id']],
        ["Certificate Hash:", Paragraph(format_hash(cert_data['metadata'].get('certificate_json_sha256', 'N/A')), small_text_style)]
    ]
    
    evidence_table = Table(evidence_data, colWidths=[2*inch, 3.5*inch])
    evidence_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (0, -1), colors.lavender),
        ('TEXTCOLOR', (0, 0), (-1, -1), colors.black),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('FONTNAME', (0, 0), (-1, -1), 'Helvetica'),
        ('FONTSIZE', (0, 0), (-1, -1), 8),
        ('GRID', (0, 0), (-1, -1), 1, colors.black),
    ]))
    
    story.append(evidence_table)
    story.append(Spacer(1, 15))
    
    # Digital Signature & QR Code
    story.append(Paragraph("Digital Signature & Verification", header_style))
    
    # Generate QR code
    qr_path = generate_qr_code(cert_data.get('metadata', {}).get('qr_payload', cert_data))
    
    # Create signature table with QR code
    sig_data = [
        ["Algorithm:", cert_data['signature']['alg']],
        ["Public Key ID:", cert_data['signature']['pubkey_id']],
        ["Signature:", Paragraph(format_hash(cert_data['signature']['sig'], 30), small_text_style)],
        ["Issuer:", f"{cert_data['issuer']['organization']} ({cert_data['issuer']['country']})"]
    ]
    
    # Two column layout: signature info and QR code
    sig_table = Table(sig_data, colWidths=[2*inch, 2.5*inch])
    sig_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (0, -1), colors.mistyrose),
        ('TEXTCOLOR', (0, 0), (-1, -1), colors.black),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('FONTNAME', (0, 0), (-1, -1), 'Helvetica'),
        ('FONTSIZE', (0, 0), (-1, -1), 8),
        ('GRID', (0, 0), (-1, -1), 1, colors.black),
    ]))
    
    # Create layout with QR code
    qr_img = Image(qr_path, width=1.5*inch, height=1.5*inch)
    
    layout_data = [[sig_table, qr_img]]
    layout_table = Table(layout_data, colWidths=[4.5*inch, 1.5*inch])
    layout_table.setStyle(TableStyle([
        ('VALIGN', (0, 0), (-1, -1), 'TOP'),
    ]))
    
    story.append(layout_table)
    story.append(Spacer(1, 20))
    
    # Footer
    footer_text = f"Generated by {cert_data['issuer']['tool_name']} v{cert_data['issuer']['tool_version']} | " \
                 f"Environment: {cert_data['environment']['os_kernel']} | " \
                 f"Operator: {cert_data['environment']['operator']}"
    story.append(Paragraph(footer_text, styles['Normal']))
    
    # Build PDF
    doc.build(story)
    
    # Clean up QR code temp file
    try:
        os.unlink(qr_path)
    except:
        pass

def main():
    print("Testing SecureWipe Wipe Certificate PDF Generation")
    print("=" * 50)
    
    # Test certificate validation first
    print("Testing certificate validation...")
    schema = load_schema()
    cert_data = create_sample_wipe_certificate()
    
    try:
        validate(instance=cert_data, schema=schema)
        print("Certificate validation: PASS")
    except ValidationError as e:
        print(f"Schema validation failed: {e.message}")
        print("Certificate validation: FAIL")
        return False
    
    # Generate PDF
    print("Generating PDF certificate...")
    output_path = "/tmp/test_wipe_certificate.pdf"
    
    try:
        create_wipe_certificate_pdf(cert_data, output_path)
        file_size = os.path.getsize(output_path)
        print(f"PDF generated successfully: {output_path}")
        print(f"File size: {file_size} bytes")
        
        print("=" * 50)
        print("Test Result: SUCCESS")
        return True
        
    except Exception as e:
        print(f"PDF generation failed: {e}")
        print("=" * 50) 
        print("Test Result: FAILED")
        return False

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
