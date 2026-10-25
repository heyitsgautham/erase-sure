/**
 * Runtime schemas for data that enters the UI from outside: Tauri command
 * results, certificate files on disk, CLI output and portal responses.
 *
 * TypeScript types stop at the Tauri boundary, so every value that crosses it
 * is parsed here before a screen uses it.
 */
import { z } from 'zod';

export const FileSystemEntrySchema = z.object({
    name: z.string(),
    path: z.string(),
    is_dir: z.boolean(),
    size: z.number().nonnegative().nullish(),
    modified: z.string().nullish(),
});

export const DirectoryListingSchema = z.object({
    entries: z.array(FileSystemEntrySchema),
    total_size: z.number().nonnegative(),
    total_items: z.number().int().nonnegative(),
});

/** What `validate_wipe_device` returns: lsblk's JSON with a criticality flag added. */
export const WipeDeviceInfoSchema = z.object({
    blockdevices: z.array(
        z
            .object({
                name: z.string().optional(),
                serial: z.string().nullish(),
                is_critical: z.boolean().optional(),
                path: z.string().optional(),
            })
            .passthrough(),
    ),
});

/** The fields the certificate list needs. Certificates carry many more; they pass through. */
export const CertificateSummarySchema = z
    .object({
        cert_id: z.string().min(1),
        cert_type: z.enum(['backup', 'wipe']),
        created_at: z.string().refine((value) => !Number.isNaN(Date.parse(value)), 'created_at is not a date'),
    })
    .passthrough();

export const VerifyResultSchema = z.object({
    schema_valid: z.boolean().nullable(),
    signature_valid: z.boolean().nullable(),
    hash_valid: z.boolean().nullable(),
    chain_valid: z.boolean().nullable(),
    cert_summary: z
        .object({
            cert_id: z.string(),
            cert_type: z.string(),
            device_model: z.string(),
            policy_method: z.string().optional(),
            destination: z.string().optional(),
        })
        .passthrough(),
    computed: z.record(z.string().nullable()),
    errors: z.array(z.string()),
});

/** One device from `securewipe discover --format json`. Older builds omit some fields. */
export const CliDeviceSchema = z
    .object({
        name: z.string().optional(),
        path: z.string().optional(),
        model: z.string().nullish(),
        serial: z.string().nullish(),
        capacity_bytes: z.number().nonnegative().optional(),
        capacity: z.number().nonnegative().optional(),
        bus: z.string().nullish(),
        mountpoints: z.array(z.string()).optional(),
        risk_level: z.enum(['CRITICAL', 'HIGH', 'SAFE']).optional(),
    })
    .refine((device) => Boolean(device.name || device.path), 'device has neither name nor path');

export const CliWipePlanSchema = z.object({
    device: z.string().optional(),
    policy: z.enum(['CLEAR', 'PURGE']).optional(),
    main_method: z.string().optional(),
    hpa_dco_clear: z.boolean().optional(),
    verification: z.object({ samples: z.number().int().positive().optional() }).optional(),
    blocked: z.boolean().optional(),
    reason: z.string().nullish(),
});

/** Explain a schema failure in one line: the first problem and where it is. */
export function describeIssues(error: z.ZodError): string {
    const issue = error.issues[0];
    const where = issue.path.length > 0 ? issue.path.join('.') : 'value';
    return `${where}: ${issue.message}`;
}
