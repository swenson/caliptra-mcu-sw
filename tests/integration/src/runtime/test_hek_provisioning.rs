// Licensed under the Apache-2.0 license

use crate::test::{start_runtime_hw_model, TestParams};
use anyhow::Result;
use mcu_hw_model::McuHwModel;
use mcu_rom_common::McuBootMilestones;
use mcu_testing_common::MCU_RUNTIME_STARTED;

const BOOT_CYCLES: u64 = 25_000_000;

#[test]
fn test_provision_first_hek() -> Result<()> {
    let mut hw = start_runtime_hw_model(TestParams {
        feature: Some("test-mcu-mbox-cmds"),
        ..Default::default()
    });

    assert!(hw
        .mci_boot_milestones()
        .contains(McuBootMilestones::FIRMWARE_BOOT_FLOW_COMPLETE));

    // This is to ensure the command happens after the mailbox responder is initialized, but it
    // doesn't change anything.
    hw.step_until(|hw| {
        hw.cycle_count() >= BOOT_CYCLES
            || MCU_RUNTIME_STARTED.load(std::sync::atomic::Ordering::Relaxed)
    });

    // wait another little bit for the mailbox to come up after the runtime
    let now = hw.cycle_count();
    hw.step_until(|hw| hw.cycle_count() >= now + 1_000_000);

    // Send an unknown command (0x0) with an invalid checksum.
    // The firmware should reject it with a mailbox failure.
    let cmd: u32 = 0x0;
    let resp = hw.mailbox_execute(cmd, &[0xaau8; 8]);
    let err_msg = format!("{}", resp.unwrap_err());
    assert!(
        !err_msg.contains("timed out"),
        "Mailbox command should fail with error, not time out. Got: {err_msg}"
    );
    Ok(())
}
