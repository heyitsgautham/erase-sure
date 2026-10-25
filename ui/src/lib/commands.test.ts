import { describe, expect, it } from 'vitest';
import { CommandResponseError, commands } from './commands';
import { installFakeTauri } from '../test/fakeTauri';

describe('commands', () => {
    it('passes arguments under the names the Rust commands expect', async () => {
        const tauri = installFakeTauri();
        tauri.on('run_securewipe', () => null);
        tauri.on('list_cert_files', () => ['/c/a.json']);
        tauri.on('read_file_content', () => '{}');
        tauri.on('file_exists', () => true);
        tauri.on('generate_pdf_for_cert', () => '/c/a.pdf');
        tauri.on('calculate_selection_size', () => 42);
        tauri.on('execute_destructive_wipe', () => null);

        await commands.runSecurewipe(['discover'], 's1');
        await commands.listCertFiles('/c');
        await commands.readFileContent('/c/a.json');
        await commands.fileExists('/c/a.pdf');
        await commands.generatePdfForCert('/c/a.json', 's2');
        await commands.calculateSelectionSize(['/a', '/b']);
        const confirmation = { device: '/dev/sdb', serial: 'X1', policy: 'purge', user_input: 'WIPE X1' };
        await commands.executeDestructiveWipe(confirmation, null);

        expect(tauri.calls).toEqual([
            { cmd: 'run_securewipe', args: { args: ['discover'], sessionId: 's1' } },
            { cmd: 'list_cert_files', args: { directory: '/c' } },
            { cmd: 'read_file_content', args: { filePath: '/c/a.json' } },
            { cmd: 'file_exists', args: { filePath: '/c/a.pdf' } },
            { cmd: 'generate_pdf_for_cert', args: { certJsonPath: '/c/a.json', sessionId: 's2' } },
            { cmd: 'calculate_selection_size', args: { paths: ['/a', '/b'] } },
            { cmd: 'execute_destructive_wipe', args: { confirmation, backupCertId: null } },
        ]);
    });

    it('returns parsed values', async () => {
        const tauri = installFakeTauri();
        tauri.on('get_home_dir', () => '/home/me');
        tauri.on('browse_folders', () => ({
            entries: [{ name: 'a.txt', path: '/home/me/a.txt', is_dir: false, size: 3, modified: null }],
            total_size: 3,
            total_items: 1,
        }));

        expect(await commands.getHomeDir()).toBe('/home/me');
        expect((await commands.browseFolders(null)).entries[0].name).toBe('a.txt');
    });

    it('rejects a response that does not match the command', async () => {
        const tauri = installFakeTauri();
        tauri.on('file_exists', () => 'yes');
        tauri.on('browse_folders', () => ({ entries: [{ name: 'a' }], total_size: 0, total_items: 1 }));

        await expect(commands.fileExists('/x')).rejects.toThrow(CommandResponseError);
        await expect(commands.fileExists('/x')).rejects.toThrow('Unexpected response from file_exists: value: Expected boolean, received string');
        await expect(commands.browseFolders('/x')).rejects.toThrow('entries.0.path: Required');
    });

    it("passes the backend's own rejection through untouched", async () => {
        const tauri = installFakeTauri();
        tauri.on('cancel_securewipe', () => {
            throw 'Session not found';
        });

        await expect(commands.cancelSecurewipe('nope')).rejects.toBe('Session not found');
    });
});
