


# 🧹 Milestone: Real Data Wiping Implementation  

## 📍 Current Context  
- Backup flow is stable (end-to-end tested).  
- Certificate signing/verification works (minor polish left).  
- Planner + Tauri UI working (non-destructive preview).  

We are now enabling **real destructive wipe commands**.  

---

## ⚠️ **Critical Safety Notes**  
1. **Development mode must never touch real system drives**. Always test with:  
   - **Loopback devices** (`losetup` on Linux)  
   - **VM virtual disks**  
   - **USB test drives**  
2. Tauri backend will require **explicit `--danger-allow-wipe` flag** (or env var `SECUREWIPE_DANGER=1`) to run destructive wipes.  
3. Wipe operations must require **interactive confirmation**:  
   - User must type `WIPE <DEVICE_SERIAL>`  
   - Without exact confirmation string → abort.  
4. Certificates must be generated for **every destructive wipe** (same chain linkage).  

---

## ✅ Milestone Deliverables  
1. **CLI Wipe Command (`securewipe wipe`)**  
   - Supports:  
     - `--device <path>` (e.g., `/dev/sdb`)  
     - `--policy clear|purge|destroy`  
     - `--sign` (optional cert signing)  
     - `--danger-allow-wipe` (safety gate)  
   - Implements wiping strategy:  
     - **Clear** = overwrite with zeros  
     - **Purge** = random pass + verify sample  
     - **Destroy** = HPA/DCO disable + multi-pass overwrite  
2. **Tauri Integration**  
   - UI: Confirmation modal (requires typed `WIPE <SERIAL>`)  
   - Backend: Only invokes wipe if `SECUREWIPE_DANGER=1` set  
   - Logs streamed to frontend  
3. **Certificate Generation**  
   - Every wipe produces a signed wipe certificate  
   - Links to backup cert if available  
   - Includes: device info, wipe policy, completion status, SHA256 logs  
4. **Testing**  
   - Run wipes on loopback devices  
   - Verify:  
     - Data is unrecoverable (`hexdump` shows zeros/random)  
     - Cert is generated and valid  
     - Portal `/verify` passes  

---

## 🛠️ Dev Testing Workflow  
1. Create a fake disk:  
   ```bash
   fallocate -l 50M /tmp/fake.img
   losetup -fP /tmp/fake.img
   # Assume it’s /dev/loop5
   ```  
2. Run wipe:  
   ```bash
   SECUREWIPE_DANGER=1 securewipe wipe --device /dev/loop5 --policy clear --danger-allow-wipe --sign
   ```  
3. Verify:  
   ```bash
   hexdump -C /dev/loop5 | head
   securewipe cert verify --file wipe_cert.json --pubkey dev_public.pem
   ```  

---

## 🚀 Milestone Completion Criteria  
- CLI can safely wipe test devices with explicit danger flag.  
- Tauri UI integrates wipe with confirmation modal.  
- Certificates generated for every destructive operation.  
- Tested on loopback & USB test drives.  
- No accidental execution on system/root disks.  

---

