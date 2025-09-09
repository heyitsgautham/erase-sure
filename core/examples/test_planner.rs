use securewipe::wipe::{plan_wipe, WipePolicy};

fn main() {
    // Test safe disk
    let plan = plan_wipe("/dev/sda", Some(WipePolicy::Purge), false, false, None, None);
    println!("Safe disk plan: {}", serde_json::to_string_pretty(&plan).unwrap());

    // Test critical disk (blocked)
    let plan = plan_wipe("/dev/sda", None, true, false, None, None);
    println!("Critical disk plan: {}", serde_json::to_string_pretty(&plan).unwrap());

    // Test with mocked controller output
    let plan = plan_wipe(
        "/dev/nvme0n1", 
        Some(WipePolicy::Clear), 
        false, 
        false, 
        Some("Security sanitize HPA DCO supported"), 
        Some("NVM Express sanitize capabilities")
    );
    println!("NVMe with capabilities: {}", serde_json::to_string_pretty(&plan).unwrap());
}