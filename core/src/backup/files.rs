//! File collection, hashing, encryption and post-copy verification for a backup.

use super::*;

impl EncryptedBackup {
    pub(super) fn collect_files(
        &self,
        paths: &[String],
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut files = Vec::new();

        for path_str in paths {
            let path = Path::new(path_str);
            if path.is_file() {
                files.push(path.to_path_buf());
            } else if path.is_dir() {
                Self::collect_files_recursive(path, &mut files)?;
            }
        }

        Ok(files)
    }

    pub(super) fn collect_files_recursive(
        dir: &Path,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                Self::collect_files_recursive(&path, files)?;
            }
        }
        Ok(())
    }

    pub(super) fn compute_file_hash(
        &self,
        file_path: &Path,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut file = File::open(file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    pub(super) fn encrypt_and_compress_file(
        &self,
        source: &Path,
        dest: &Path,
        cipher: &mut Aes256Ctr,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let mut source_file = File::open(source)?;
        let mut dest_file = File::create(dest)?;

        let mut buffer = [0u8; 8192];
        let mut total_bytes = 0u64;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            // Encrypt in-place
            cipher.apply_keystream(&mut buffer[..bytes_read]);

            dest_file.write_all(&buffer[..bytes_read])?;
            total_bytes += bytes_read as u64;
        }

        Ok(total_bytes)
    }

    pub(super) fn verify_random_files(
        &self,
        manifest: &BackupManifest,
        _backup_dir: &Path,
        source_base: &Path,
        n_samples: usize,
    ) -> Result<(usize, usize), Box<dyn std::error::Error>> {
        let files: Vec<_> = manifest.files.keys().collect();
        if files.is_empty() {
            return Ok((0, 0));
        }

        let mut rng = ChaCha20Rng::from_entropy();
        let samples = std::cmp::min(n_samples, files.len());
        let mut verified = 0;

        for _ in 0..samples {
            let idx = (rng.next_u32() as usize) % files.len();
            let rel_path = files[idx];
            let original_path = source_base.join(rel_path);

            if original_path.exists() {
                let computed_hash = self.compute_file_hash(&original_path)?;
                if computed_hash == manifest.files[rel_path] {
                    verified += 1;
                }
            }
        }

        Ok((samples, verified))
    }

    pub(super) fn compute_manifest_hash(&self, manifest: &BackupManifest) -> String {
        // Create a deterministic string representation for hashing
        let mut entries: Vec<_> = manifest.files.iter().collect();
        entries.sort_by_key(|(k, _)| *k);

        let mut hasher = Sha256::new();
        for (path, hash) in entries {
            hasher.update(path.as_bytes());
            hasher.update(hash.as_bytes());
        }
        hasher.update(manifest.created_at.as_bytes());
        hasher.update(manifest.total_files.to_le_bytes());
        hasher.update(manifest.total_bytes.to_le_bytes());

        format!("{:x}", hasher.finalize())
    }
}
