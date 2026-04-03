pub mod cpu;
pub mod errors;
pub mod igpu;
pub mod platform;
pub mod ram;
pub mod types;

pub use errors::HwProbeError;
pub use types::*;

pub fn probe_all() -> Result<HardwareProfile, HwProbeError> {
    let cpu = cpu::probe_cpu()?;
    let igpu = igpu::probe_igpu()?;
    let ram = ram::probe_ram()?;
    let platform = platform::probe_platform()?;

    Ok(HardwareProfile {
        cpu,
        igpu,
        ram,
        dgpu: None,
        platform,
    })
}
