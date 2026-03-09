use crate::core::providers::ProviderRegistry;
use crate::ui::UiManager;

#[cfg(feature = "native")]
use crate::assets::AudioManager;

use super::GoudGame;

impl GoudGame {
    /// Returns a reference to the provider registry.
    #[inline]
    pub fn providers(&self) -> &ProviderRegistry {
        &self.providers
    }

    /// Returns a reference to the audio manager, if available.
    #[cfg(feature = "native")]
    #[inline]
    pub fn audio_manager(&self) -> Option<&AudioManager> {
        self.audio_manager.as_ref()
    }

    /// Returns a mutable reference to the audio manager, if available.
    #[cfg(feature = "native")]
    #[inline]
    pub fn audio_manager_mut(&mut self) -> Option<&mut AudioManager> {
        self.audio_manager.as_mut()
    }

    /// Returns a reference to the UI manager.
    #[inline]
    pub fn ui_manager(&self) -> &UiManager {
        &self.ui_manager
    }

    /// Returns a mutable reference to the UI manager.
    #[inline]
    pub fn ui_manager_mut(&mut self) -> &mut UiManager {
        &mut self.ui_manager
    }
}
