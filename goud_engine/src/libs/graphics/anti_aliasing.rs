/// Runtime anti-aliasing mode selection for 3D rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AntiAliasingMode {
    /// Disable all anti-aliasing.
    #[default]
    Off = 0,
    /// Use hardware MSAA only.
    Msaa = 1,
    /// Use FXAA post-processing only.
    Fxaa = 2,
    /// Apply both MSAA and FXAA.
    MsaaFxaa = 3,
}

impl AntiAliasingMode {
    /// Returns true when the mode includes MSAA.
    pub const fn uses_msaa(self) -> bool {
        matches!(self, Self::Msaa | Self::MsaaFxaa)
    }

    /// Returns true when the mode includes FXAA.
    pub const fn uses_fxaa(self) -> bool {
        matches!(self, Self::Fxaa | Self::MsaaFxaa)
    }
}
