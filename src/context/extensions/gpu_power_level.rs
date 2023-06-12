#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum GpuPowerLevel {
    PreferIntegrated,
    PreferDiscrete,
}
