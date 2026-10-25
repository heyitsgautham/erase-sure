import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import BackupConfigForm from './BackupConfigForm';
import BackupHeader from './BackupHeader';
import BackupResultCard from './BackupResultCard';
import SourceDeviceCard from './SourceDeviceCard';
import { installFakeTauri } from '../../test/fakeTauri';
import { renderWithApp } from '../../test/render';
import { safeUsb, systemNvme } from '../../test/devices';
import type { Device } from '../../contexts/AppContext';

describe('BackupHeader', () => {
    it('lists the four protections', () => {
        render(<BackupHeader />);

        for (const feature of [
            'AES-256-CTR Encryption',
            'PBKDF2 Key Derivation',
            'HMAC-SHA256 Integrity',
            'N=5 Sample Verification',
        ]) {
            expect(screen.getByText(feature)).toBeInTheDocument();
        }
    });
});

describe('SourceDeviceCard', () => {
    it('shows model, capacity, serial and risk', () => {
        render(<SourceDeviceCard device={systemNvme} />);

        expect(screen.getByText('Samsung SSD 980')).toBeInTheDocument();
        expect(screen.getByText('465.8 GB')).toBeInTheDocument();
        expect(screen.getByText('S64DNX0R123456')).toBeInTheDocument();
        expect(screen.getByText('CRITICAL')).toHaveClass('risk-badge', 'critical');
    });
});

/** Holds the form's state the way the Backup screen does. */
function Form({ device, onBrowse = () => {} }: { device: Device; onBrowse?: () => void }) {
    const [destination, setDestination] = useState('~/SecureWipe/backups');
    const [useDefaultPaths, setUseDefaultPaths] = useState(true);
    const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
    const [signKeyPath, setSignKeyPath] = useState('');
    return (
        <>
            <BackupConfigForm
                device={device}
                destination={destination}
                onDestinationChange={setDestination}
                onBrowseDestination={onBrowse}
                useDefaultPaths={useDefaultPaths}
                onUseDefaultPathsChange={setUseDefaultPaths}
                selectedFiles={selectedFiles}
                onSelectedFilesChange={setSelectedFiles}
                signKeyPath={signKeyPath}
                onSignKeyPathChange={setSignKeyPath}
            />
            <output data-testid="values">{JSON.stringify({ destination, signKeyPath, useDefaultPaths })}</output>
        </>
    );
}

describe('BackupConfigForm', () => {
    it('edits the destination and signing key, and asks to browse', async () => {
        const user = userEvent.setup();
        const onBrowse = vi.fn();
        renderWithApp(<Form device={safeUsb} onBrowse={onBrowse} />);

        const destination = screen.getByPlaceholderText('~/SecureWipe/backups');
        await user.clear(destination);
        await user.type(destination, '/mnt/usb');
        await user.type(screen.getByPlaceholderText('Path to private key for certificate signing'), '/keys/k.pem');
        await user.click(screen.getByRole('button', { name: /Browse$/ }));

        expect(JSON.parse(screen.getByTestId('values').textContent ?? '{}')).toMatchObject({
            destination: '/mnt/usb',
            signKeyPath: '/keys/k.pem',
        });
        expect(onBrowse).toHaveBeenCalledTimes(1);
    });

    it('keeps a removable device on the default selection without the system disk notice', () => {
        renderWithApp(<Form device={safeUsb} />);

        expect(screen.queryByText('System Disk Backup Mode')).not.toBeInTheDocument();
        expect(screen.queryByRole('checkbox')).not.toBeInTheDocument();
    });

    it('limits a system disk to user folders until the user opts out, then offers the file browser', async () => {
        const user = userEvent.setup();
        const tauri = installFakeTauri();
        tauri.on('browse_folders', () => ({ entries: [], total_size: 0, total_items: 0 }));
        renderWithApp(<Form device={systemNvme} />);

        expect(screen.getByText('System Disk Backup Mode')).toBeInTheDocument();
        expect(screen.getByText('Documents folder')).toBeInTheDocument();

        await user.click(screen.getByRole('checkbox'));
        expect(screen.getByText('Custom Additional Files/Folders')).toBeInTheDocument();

        await user.click(screen.getByRole('button', { name: /Browse Files/ }));
        expect(screen.getByText('Select Files and Folders for Backup')).toBeInTheDocument();

        await user.click(screen.getByRole('button', { name: /Hide Browser/ }));
        expect(screen.queryByText('Select Files and Folders for Backup')).not.toBeInTheDocument();
    });
});

describe('BackupResultCard', () => {
    const result = {
        backup_path: '/backups/1234',
        manifest_path: '/backups/1234/manifest.json',
        certificate_json_path: '/certs/1234.json',
        certificate_pdf_path: '/certs/1234.pdf',
        integrity_checks: 5,
    };

    it('shows where the backup went and links every file it produced', () => {
        renderWithApp(<BackupResultCard result={result} onViewCertificates={() => {}} />);

        expect(screen.getAllByText('/backups/1234')).toHaveLength(2);
        expect(screen.getByText('5 samples verified')).toBeInTheDocument();
        expect(screen.getByText('Backup Manifest')).toBeInTheDocument();
        expect(screen.getByText('Backup Certificate (JSON)')).toBeInTheDocument();
        expect(screen.getByText('Backup Certificate (PDF)')).toBeInTheDocument();
    });

    it('hides certificate links the CLI did not report and opens the certificate list', async () => {
        const onViewCertificates = vi.fn();
        renderWithApp(
            <BackupResultCard
                result={{ ...result, certificate_json_path: undefined, certificate_pdf_path: undefined }}
                onViewCertificates={onViewCertificates}
            />,
        );

        expect(screen.queryByText('Backup Certificate (JSON)')).not.toBeInTheDocument();
        await userEvent.click(screen.getByRole('button', { name: /View All Certificates/ }));
        expect(onViewCertificates).toHaveBeenCalledTimes(1);
    });
});
