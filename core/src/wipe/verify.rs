//! Post-wipe verification by sampling random 512-byte sectors.

use super::*;

impl NistAlignedWipe {
    pub(super) fn verify_wipe(
        &self,
        device: &str,
        sample_count: usize,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        info!(
            device,
            sample_count, "Verifying wipe with random sector samples"
        );

        let mut file = OpenOptions::new().read(true).open(device)?;

        file.seek(SeekFrom::End(0))?;
        let device_size = file.stream_position()?;

        let sector_count = device_size / 512;
        if sector_count == 0 || sample_count == 0 {
            return Err(format!(
                "Cannot verify {}: size {} bytes holds no full 512-byte sector",
                device, device_size
            )
            .into());
        }

        let mut rng = rand::thread_rng();
        let mut verified_count = 0;

        for _ in 0..sample_count {
            let offset = (rng.next_u64() % sector_count) * 512;

            file.seek(SeekFrom::Start(offset))?;
            let mut buffer = [0u8; 512];
            file.read_exact(&mut buffer)?;

            // Check if sector appears to be wiped (mostly zeros or random-looking)
            let zero_count = buffer.iter().filter(|&&b| b == 0).count();
            let is_likely_wiped = zero_count > 400 || self.appears_random(&buffer);

            if is_likely_wiped {
                verified_count += 1;
            }
        }

        // Consider verification passed if >95% of samples look wiped
        let success_threshold = (sample_count * 95) / 100;
        let passed = verified_count >= success_threshold;

        info!(
            device,
            verified_count, sample_count, passed, "Verification sampling complete"
        );

        Ok(passed)
    }

    pub(super) fn appears_random(&self, data: &[u8]) -> bool {
        // Simple randomness check: count byte-to-byte transitions.
        // More than 30% transitions suggests random data.
        let transitions = data.windows(2).filter(|w| w[0] != w[1]).count();
        (transitions * 100) / data.len() > 30
    }
}
