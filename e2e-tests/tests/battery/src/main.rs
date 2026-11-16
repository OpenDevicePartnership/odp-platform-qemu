//! E2E test: Battery service GetBst and GetPsr via FF-A Direct Request v2.
//!
//! SPDX-License-Identifier: MIT
//!

#![no_main]
#![no_std]

extern crate alloc;

use test_support::{run_tests, E2eContext, BATTERY_UUID};
use uefi::prelude::*;

/// EC Battery GetBst opcode (see battery-service-relay `BatteryCmd::GetBst = 2`).
const EC_BAT_GET_BST: u8 = 0x02;
/// EC Battery GetPsr opcode (see battery-service-relay `BatteryCmd::GetPsr = 3`).
const EC_BAT_GET_PSR: u8 = 0x03;

/// EC dev-qemu registers a single fuel gauge as battery 0 by default. A second
/// pack requires the EC's opt-in `dual-fuel-gauge` feature.
const BATTERY_ID: u8 = 0;

// EC dev-qemu MockFuelGauge BST values, mapped by the EC's compute_bst. These
// are fixed mock constants, asserted exactly.
const EXPECT_STATE_DISCHARGING: u32 = 0x1; // ACPI BatteryState::DISCHARGING = 1<<0
const EXPECT_PRESENT_RATE: u32 = 1500; // |−1500 mA| discharge
const EXPECT_REMAINING_CAPACITY: u32 = 2304; // mAh
const EXPECT_PRESENT_VOLTAGE: u32 = 11850; // mV (3 × 3950)

// ACPI _PSR Offline. The EC's mock adapter starts unplugged and only the
// `time-alarm-power-input` build plugs it in (via GPIO2).
const EXPECT_PSR_OFFLINE: u32 = 0;

#[entry]
fn main() -> Status {
    run_tests(test_battery_command_family)
}

fn test_battery_command_family(ctx: &mut E2eContext) {
    test_battery_get_bst(ctx);
    test_battery_get_psr(ctx);
}

fn test_battery_get_bst(ctx: &mut E2eContext) {
    const NAME: &str = "battery_get_bst";
    let Some(resp_payload) = ctx.send_command(NAME, &BATTERY_UUID, EC_BAT_GET_BST, &[BATTERY_ID])
    else {
        return;
    };

    // Response layout (4 LE u32): state@0, rate@4, capacity@8, voltage@12.
    let state = resp_payload.u32_at(0);
    let rate = resp_payload.u32_at(4);
    let capacity = resp_payload.u32_at(8);
    let voltage = resp_payload.u32_at(12);

    log::info!(
        "  GetBst[id={}]: state={:#x} rate={} capacity={} voltage={}",
        BATTERY_ID,
        state,
        rate,
        capacity,
        voltage,
    );

    // Exact equality: the mock is discharging only (DISCHARGING set, CHARGING /
    // CRITICAL / CHARGE_LIMITING clear), so the ACPI state word is exactly 0x1.
    if state != EXPECT_STATE_DISCHARGING {
        ctx.fail(NAME, "EC BST state != DISCHARGING-only (0x1)");
        return;
    }
    if rate != EXPECT_PRESENT_RATE {
        ctx.fail(NAME, "present_rate != EC mock value");
        return;
    }
    if capacity != EXPECT_REMAINING_CAPACITY {
        ctx.fail(NAME, "remaining_capacity != EC mock value");
        return;
    }
    if voltage != EXPECT_PRESENT_VOLTAGE {
        ctx.fail(NAME, "present_voltage != EC mock value");
        return;
    }
    ctx.pass(NAME);
}

fn test_battery_get_psr(ctx: &mut E2eContext) {
    const NAME: &str = "battery_get_psr";
    let Some(resp_payload) = ctx.send_command(NAME, &BATTERY_UUID, EC_BAT_GET_PSR, &[BATTERY_ID])
    else {
        return;
    };
    // Response layout: power source (LE u32) @0.
    let power_source = resp_payload.u32_at(0);
    log::info!("  GetPsr[id={}]: power_source={}", BATTERY_ID, power_source);

    if power_source != EXPECT_PSR_OFFLINE {
        ctx.fail(NAME, "_PSR != Offline without the power-input build");
        return;
    }
    ctx.pass(NAME);
}
