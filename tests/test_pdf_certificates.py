#!/usr/bin/env python3
"""
Test PDF certificate generation with the new audit-ready backup schema.
Tests both JSON validation and PDF output formatting.
"""

import json
import tempfile
import os
from datetime import datetime
from pathlib import Path
import jsonschema
from reportlab.lib.pagesizes import letter, A4
from reportlab.platypus import SimpleDocTemplate, Table, TableStyle, Paragraph, Spacer, Image
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.units import inch
from reportlab.lib import colors
from reportlab.lib.enums import TA_LEFT, TA_CENTER, TA_RIGHT
import qrcode
from io import BytesIO
import base64


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


class BackupCertificatePDFGenerator:
    """Generate PDF certificates from backup certificate JSON matching the new schema."""
    
    def __init__(self, schema_path: str = None):
        if schema_path is None:
            # Default to the schema in the project
            self.schema_path = Path(__file__).parent.parent / "certs" / "schemas" / "backup_schema.json"
        else:
            self.schema_path = Path(schema_path)
            
        with open(self.schema_path, 'r') as f:
            self.schema = json.load(f)
    
    def validate_certificate(self, cert_data: dict) -> bool:
        """Validate certificate against the JSON schema."""
        try:
            jsonschema.validate(cert_data, self.schema)
            return True
        except jsonschema.exceptions.ValidationError as e:
            print(f"Schema validation failed: {e}")
            return False
    
    def generate_qr_code(self, data: str) -> Image:
        """Generate QR code for certificate verification."""
        qr = qrcode.QRCode(version=1, box_size=10, border=5)
        qr.add_data(data)
        qr.make(fit=True)
        
        img = qr.make_image(fill_color="black", back_color="white")
        
        # Convert to reportlab Image
        buffer = BytesIO()
        img.save(buffer, format='PNG')
        buffer.seek(0)
        
        return Image(buffer, width=1.5*inch, height=1.5*inch)
    
    def create_certificate_pdf(self, cert_data: dict, output_path: str, skip_validation: bool = False) -> bool:
        """Generate PDF certificate from validated JSON data."""
        
        # Validate first unless explicitly skipped
        if not skip_validation:
            if not self.validate_certificate(cert_data):
                print("⚠️  Certificate validation failed, but proceeding with PDF generation anyway")
                # Don't return False - proceed with PDF generation for unsigned certs
        
        doc = SimpleDocTemplate(output_path, pagesize=A4, 
                              rightMargin=72, leftMargin=72,
                              topMargin=72, bottomMargin=18)
        
        styles = getSampleStyleSheet()
        title_style = ParagraphStyle(
            'CustomTitle',
            parent=styles['Heading1'],
            fontSize=24,
            spaceAfter=30,
            alignment=1,  # Center
            textColor=colors.HexColor('#1f4e79')
        )
        
        heading_style = ParagraphStyle(
            'CustomHeading',
            parent=styles['Heading2'],
            fontSize=16,
            spaceAfter=12,
            textColor=colors.HexColor('#2f5f8f')
        )
        
        story = []
        
        # Header
        story.append(Paragraph("SecureWipe Backup Certificate", title_style))
        story.append(Paragraph(f"Certificate ID: {cert_data['cert_id']}", styles['Normal']))
        story.append(Paragraph(f"Version: {cert_data['certificate_version']}", styles['Normal']))
        story.append(Paragraph(f"Issued: {cert_data['created_at']}", styles['Normal']))
        story.append(Spacer(1, 20))
        
        # Result Badge
        result_color = colors.green if cert_data['result'] == 'PASS' else colors.red
        result_style = ParagraphStyle('Result', parent=styles['Normal'], 
                                    fontSize=18, textColor=result_color, alignment=1)
        story.append(Paragraph(f"<b>RESULT: {cert_data['result']}</b>", result_style))
        story.append(Spacer(1, 20))
        
        # Issuer Information
        story.append(Paragraph("Issuer Information", heading_style))
        issuer_data = [
            ['Organization', cert_data['issuer']['organization']],
            ['Tool', f"{cert_data['issuer']['tool_name']} v{cert_data['issuer']['tool_version']}"],
            ['Country', cert_data['issuer'].get('country', 'N/A')]
        ]
        issuer_table = Table(issuer_data, colWidths=[2*inch, 4*inch])
        issuer_table.setStyle(TableStyle([
            ('BACKGROUND', (0, 0), (0, -1), colors.lightgrey),
            ('TEXTCOLOR', (0, 0), (-1, -1), colors.black),
            ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
            ('FONTNAME', (0, 0), (-1, -1), 'Helvetica'),
            ('FONTSIZE', (0, 0), (-1, -1), 10),
            ('BOTTOMPADDING', (0, 0), (-1, -1), 12),
            ('GRID', (0, 0), (-1, -1), 1, colors.black)
        ]))
        story.append(issuer_table)
        story.append(Spacer(1, 15))
        
        # Device Information
        story.append(Paragraph("Device Information", heading_style))
        device_data = [
            ['Model', cert_data['device']['model']],
            ['Serial', cert_data['device']['serial']],
            ['Bus Type', cert_data['device']['bus']],
            ['Capacity', f"{cert_data['device']['capacity_bytes']:,} bytes"],
            ['Path', cert_data['device'].get('path', 'N/A')]
        ]
        device_table = Table(device_data, colWidths=[2*inch, 4*inch])
        device_table.setStyle(TableStyle([
            ('BACKGROUND', (0, 0), (0, -1), colors.lightgrey),
            ('TEXTCOLOR', (0, 0), (-1, -1), colors.black),
            ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
            ('FONTNAME', (0, 0), (-1, -1), 'Helvetica'),
            ('FONTSIZE', (0, 0), (-1, -1), 10),
            ('BOTTOMPADDING', (0, 0), (-1, -1), 12),
            ('GRID', (0, 0), (-1, -1), 1, colors.black)
        ]))
        story.append(device_table)
        story.append(Spacer(1, 15))
        
        # Backup Summary
        story.append(Paragraph("Backup Summary", heading_style))
        backup_data = [
            ['Files Count', f"{cert_data['files_summary']['count']:,}"],
            ['Personal Data', f"{cert_data['files_summary']['personal_bytes']:,} bytes"],
            ['Destination Type', cert_data['destination']['type'].upper()],
            ['Destination Label', cert_data['destination'].get('label', 'N/A')],
            ['File System', cert_data['destination'].get('fs', 'N/A')]
        ]
        backup_table = Table(backup_data, colWidths=[2*inch, 4*inch])
        backup_table.setStyle(TableStyle([
            ('BACKGROUND', (0, 0), (0, -1), colors.lightgrey),
            ('TEXTCOLOR', (0, 0), (-1, -1), colors.black),
            ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
            ('FONTNAME', (0, 0), (-1, -1), 'Helvetica'),
            ('FONTSIZE', (0, 0), (-1, -1), 10),
            ('BOTTOMPADDING', (0, 0), (-1, -1), 12),
            ('GRID', (0, 0), (-1, -1), 1, colors.black)
        ]))
        story.append(backup_table)
        story.append(Spacer(1, 15))
        
        # Security & Verification
        story.append(Paragraph("Security & Verification", heading_style))
        security_data = [
            ['Encryption', cert_data['crypto']['alg']],
            ['Key Management', cert_data['crypto']['key_management'].replace('_', ' ').title()],
            ['Verification Strategy', cert_data['verification']['strategy'].replace('_', ' ').title()],
            ['Verification Failures', str(cert_data['verification']['failures'])],
            ['Policy', f"{cert_data['policy']['name']} v{cert_data['policy']['version']}"]
        ]
        
        # Add coverage info if present
        if 'coverage' in cert_data['verification']:
            coverage = cert_data['verification']['coverage']
            if coverage['mode'] == 'percent':
                coverage_str = f"{coverage['percent']}%"
            else:
                coverage_str = f"{coverage['samples']} samples"
            security_data.append(['Verification Coverage', coverage_str])
        
        security_table = Table(security_data, colWidths=[2*inch, 4*inch])
        security_table.setStyle(TableStyle([
            ('BACKGROUND', (0, 0), (0, -1), colors.lightgrey),
            ('TEXTCOLOR', (0, 0), (-1, -1), colors.black),
            ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
            ('FONTNAME', (0, 0), (-1, -1), 'Helvetica'),
            ('FONTSIZE', (0, 0), (-1, -1), 10),
            ('BOTTOMPADDING', (0, 0), (-1, -1), 12),
            ('GRID', (0, 0), (-1, -1), 1, colors.black)
        ]))
        story.append(security_table)
        story.append(Spacer(1, 15))
        
        # Environment
        story.append(Paragraph("Environment", heading_style))
        env_data = [
            ['Operator', cert_data['environment']['operator']],
            ['OS Kernel', cert_data['environment']['os_kernel']],
            ['Tool Version', cert_data['environment']['tool_version']],
            ['Containerized', str(cert_data['environment'].get('containerized', 'N/A'))]
        ]
        env_table = Table(env_data, colWidths=[2*inch, 4*inch])
        env_table.setStyle(TableStyle([
            ('BACKGROUND', (0, 0), (0, -1), colors.lightgrey),
            ('TEXTCOLOR', (0, 0), (-1, -1), colors.black),
            ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
            ('FONTNAME', (0, 0), (-1, -1), 'Helvetica'),
            ('FONTSIZE', (0, 0), (-1, -1), 10),
            ('BOTTOMPADDING', (0, 0), (-1, -1), 12),
            ('GRID', (0, 0), (-1, -1), 1, colors.black)
        ]))
        story.append(env_table)
        story.append(Spacer(1, 20))
        
        # QR Code for verification
        story.append(Paragraph("Certificate Verification", heading_style))
        
        # Create QR data - either verify_url or cert_id
        qr_data = cert_data.get('verify_url', f"cert_id:{cert_data['cert_id']}")
        qr_image = self.generate_qr_code(qr_data)
        
        # Center the QR code
        qr_table = Table([[qr_image]], colWidths=[6*inch])
        qr_table.setStyle(TableStyle([('ALIGN', (0, 0), (-1, -1), 'CENTER')]))
        story.append(qr_table)
        
        story.append(Paragraph("Scan QR code to verify certificate authenticity", styles['Normal']))
        
        # Add verification URL if available
        if 'verify_url' in cert_data:
            verify_data = [
                ['Verification Portal', create_clickable_url(cert_data['verify_url'], None, styles['Normal'])]
            ]
            verify_table = Table(verify_data, colWidths=[2*inch, 4*inch])
            verify_table.setStyle(TableStyle([
                ('BACKGROUND', (0, 0), (0, -1), colors.lightgrey),
                ('TEXTCOLOR', (0, 0), (-1, -1), colors.black),
                ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
                ('FONTNAME', (0, 0), (-1, -1), 'Helvetica'),
                ('FONTSIZE', (0, 0), (-1, -1), 10),
                ('BOTTOMPADDING', (0, 0), (-1, -1), 12),
                ('GRID', (0, 0), (-1, -1), 1, colors.black)
            ]))
            story.append(Spacer(1, 10))
            story.append(verify_table)
        
        # Signature info
        story.append(Spacer(1, 20))
        story.append(Paragraph("Digital Signature", heading_style))
        
        # Handle null/missing signature (unsigned certificates)
        if cert_data.get('signature') is None:
            sig_data = [
                ['Status', 'UNSIGNED CERTIFICATE'],
                ['Note', 'This certificate has not been digitally signed'],
                ['Verification', 'Manual verification required']
            ]
        else:
            sig_data = [
                ['Algorithm', cert_data['signature']['alg']],
                ['Public Key ID', cert_data['signature']['pubkey_id']],
                ['Signature', cert_data['signature']['sig'][:50] + '...']  # Truncated for display
            ]
        
        sig_table = Table(sig_data, colWidths=[2*inch, 4*inch])
        sig_table.setStyle(TableStyle([
            ('BACKGROUND', (0, 0), (0, -1), colors.lightgrey),
            ('TEXTCOLOR', (0, 0), (-1, -1), colors.black),
            ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
            ('FONTNAME', (0, 0), (-1, -1), 'Helvetica'),
            ('FONTSIZE', (0, 0), (-1, -1), 9),
            ('BOTTOMPADDING', (0, 0), (-1, -1), 12),
            ('GRID', (0, 0), (-1, -1), 1, colors.black)
        ]))
        story.append(sig_table)
        
        # Build PDF
        doc.build(story)
        return True


def test_pdf_generation():
    """Test PDF generation with sample data matching the new schema."""
    
    # Sample certificate data matching the new schema
    sample_cert = {
        "cert_type": "backup",
        "cert_id": "TEST_BCK_2024_001",
        "certificate_version": "v1.0.0",
        "created_at": "2024-01-15T10:30:00+05:30",
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
            "firmware": "5B2QGXA7"
        },
        "files_summary": {
            "count": 2847,
            "personal_bytes": 52428800,
            "included_paths": ["/home/user/Documents", "/home/user/Pictures"],
            "excluded_paths": ["/home/user/.cache"]
        },
        "destination": {
            "type": "usb",
            "label": "BACKUP_USB_2024",
            "fs": "exfat",
            "mountpoint": "/media/backup"
        },
        "crypto": {
            "alg": "AES-256-CTR",
            "manifest_sha256": "d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2",
            "key_management": "ephemeral_session_key"
        },
        "verification": {
            "strategy": "sampled_files",
            "coverage": {
                "mode": "percent",
                "percent": 15.5
            },
            "failures": 0,
            "notes": "All sampled files verified successfully"
        },
        "policy": {
            "name": "NIST SP 800-88 Rev.1",
            "version": "2023.12"
        },
        "result": "PASS",
        "environment": {
            "operator": "Automated",
            "os_kernel": "Linux 6.8.0-35-generic",
            "tool_version": "v2.1.0",
            "containerized": False
        },
        "exceptions": {
            "text": "None"
        },
        "signature": {
            "alg": "Ed25519",
            "pubkey_id": "sih_root_v1",
            "sig": "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef=="
        },
        "metadata": {
            "certificate_json_sha256": "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678",
            "qr_payload": {
                "cert_id": "TEST_BCK_2024_001",
                "issued_at": "2024-01-15T10:30:00+05:30",
                "device_model": "Samsung SSD 980 PRO 1TB",
                "result": "PASS",
                "verify_url": "https://verify.securewipe.org/cert/TEST_BCK_2024_001"
            }
        },
        "verify_url": "https://verify.securewipe.org/cert/TEST_BCK_2024_001"
    }
    
    # Create PDF generator
    generator = BackupCertificatePDFGenerator()
    
    # Test validation
    print("Testing certificate validation...")
    is_valid = generator.validate_certificate(sample_cert)
    print(f"Certificate validation: {'PASS' if is_valid else 'FAIL'}")
    
    if not is_valid:
        return False
    
    # Generate PDF
    print("Generating PDF certificate...")
    output_path = "/tmp/test_backup_certificate.pdf"
    
    try:
        success = generator.create_certificate_pdf(sample_cert, output_path)
        if success:
            print(f"PDF generated successfully: {output_path}")
            print(f"File size: {os.path.getsize(output_path)} bytes")
            return True
        else:
            print("PDF generation failed")
            return False
    except Exception as e:
        print(f"Error generating PDF: {e}")
        return False


if __name__ == "__main__":
    print("Testing SecureWipe PDF Certificate Generation")
    print("=" * 50)
    
    success = test_pdf_generation()
    
    print("=" * 50)
    print(f"Test Result: {'SUCCESS' if success else 'FAILED'}")