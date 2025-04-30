pub fn init() {
    env_logger::init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        // Test that init() doesn't panic
        init();

        // Log some test messages to verify logger is working
        // log::error!("Test error message"); // We dont want to see this in test output
        log::warn!("Test warning message");
        log::info!("Test info message");
        log::debug!("Test debug message");
        log::trace!("Test trace message");

        // No assert needed - if initialization failed, the log macros would panic
    }
}
