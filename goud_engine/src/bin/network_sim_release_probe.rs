use goud_engine::core::error::ERR_INVALID_STATE;
use goud_engine::core::providers::network_types::NetworkSimulationConfig;
use goud_engine::ffi::context::GOUD_INVALID_CONTEXT_ID;
use goud_engine::ffi::network::{goud_network_clear_simulation, goud_network_set_simulation};

fn main() {
    let config = NetworkSimulationConfig {
        one_way_latency_ms: 5,
        jitter_ms: 1,
        packet_loss_percent: 2.5,
    };

    let set_code = goud_network_set_simulation(GOUD_INVALID_CONTEXT_ID, 42, config);
    let clear_code = goud_network_clear_simulation(GOUD_INVALID_CONTEXT_ID, 42);

    if set_code == ERR_INVALID_STATE && clear_code == ERR_INVALID_STATE {
        return;
    }

    eprintln!(
        "expected release stubs to return ERR_INVALID_STATE, got set={} clear={}",
        set_code, clear_code
    );
    std::process::exit(1);
}
