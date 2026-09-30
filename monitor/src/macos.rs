//! macOS backend: Metal device memory, kernel memory pressure, thermal state, power.

use std::ffi::c_void;

use core_foundation::base::{CFType, TCFType};
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::number::CFNumber;
use core_foundation::string::{CFString, CFStringRef};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_foundation::NSProcessInfo;
use objc2_metal::{MTLCreateSystemDefaultDevice, MTLDevice};

use crate::OsReadings;

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOPSCopyPowerSourcesInfo() -> *const c_void;
    fn IOPSCopyPowerSourcesList(blob: *const c_void) -> *const c_void;
    fn IOPSGetPowerSourceDescription(blob: *const c_void, ps: *const c_void) -> CFDictionaryRef;
    fn IOPSGetProvidingPowerSourceType(blob: *const c_void) -> CFStringRef;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFArrayGetCount(array: *const c_void) -> isize;
    fn CFArrayGetValueAtIndex(array: *const c_void, idx: isize) -> *const c_void;
    fn CFRelease(cf: *const c_void);
}

pub struct Reader {
    device: Option<Retained<ProtocolObject<dyn MTLDevice>>>,
}

impl Reader {
    pub fn new() -> Self {
        Self {
            device: MTLCreateSystemDefaultDevice(),
        }
    }

    pub fn read(&mut self) -> OsReadings {
        let (gpu_alloc_bytes, gpu_budget_bytes) = match &self.device {
            Some(d) => (
                Some(d.currentAllocatedSize() as u64),
                Some(d.recommendedMaxWorkingSetSize()),
            ),
            None => (None, None),
        };
        let info = NSProcessInfo::processInfo();
        let (on_battery, battery_pct) = power_state();
        OsReadings {
            gpu_alloc_bytes,
            gpu_budget_bytes,
            sys_mem_pressure: sysctl_u32(c"kern.memorystatus_vm_pressure_level"),
            sys_mem_free_pct: sysctl_u32(c"kern.memorystatus_level"),
            thermal_state: Some(info.thermalState().0 as u32),
            on_battery,
            battery_pct,
            low_power_mode: Some(info.isLowPowerModeEnabled()),
        }
    }
}

fn sysctl_u32(name: &std::ffi::CStr) -> Option<u32> {
    let mut value: i32 = 0;
    let mut len = size_of::<i32>();
    let rc = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            (&raw mut value).cast(),
            &mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    (rc == 0).then_some(value as u32)
}

/// Returns (on battery, battery percent). A desktop Mac without a battery reports
/// the providing source but no percentage.
fn power_state() -> (Option<bool>, Option<u32>) {
    unsafe {
        let blob = IOPSCopyPowerSourcesInfo();
        if blob.is_null() {
            return (None, None);
        }

        let providing = IOPSGetProvidingPowerSourceType(blob);
        let on_battery = (!providing.is_null())
            .then(|| CFString::wrap_under_get_rule(providing).to_string() == "Battery Power");

        let mut pct = None;
        let list = IOPSCopyPowerSourcesList(blob);
        if !list.is_null() {
            for i in 0..CFArrayGetCount(list) {
                let desc = IOPSGetPowerSourceDescription(blob, CFArrayGetValueAtIndex(list, i));
                if desc.is_null() {
                    continue;
                }
                let desc: CFDictionary<CFString, CFType> = CFDictionary::wrap_under_get_rule(desc);
                let num = |key: &str| {
                    desc.find(CFString::new(key))
                        .and_then(|v| v.downcast::<CFNumber>())
                        .and_then(|n| n.to_i64())
                };
                if let (Some(cur), Some(max)) = (num("Current Capacity"), num("Max Capacity"))
                    && max > 0
                {
                    pct = Some((cur * 100 / max) as u32);
                    break;
                }
            }
            CFRelease(list);
        }
        CFRelease(blob);
        (on_battery, pct)
    }
}
