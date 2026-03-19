#include <goud/goud.hpp>

#include <cstdio>
#include <cstdlib>

int main() {
    int status = 0;

    // Create engine configuration via RAII wrapper
    goud::EngineConfig config = goud::EngineConfig::create(&status);
    if (status != SUCCESS) {
        std::fprintf(stderr, "Failed to create engine config (status %d)\n", status);
        return EXIT_FAILURE;
    }

    config.setTitle("CMake Example");
    config.setSize(320, 240);

    std::printf("CMake example built and linked successfully!\n");

    // config is cleaned up automatically via RAII
    return EXIT_SUCCESS;
}
