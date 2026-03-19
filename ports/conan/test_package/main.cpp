#include <goud/goud.hpp>
#include <cstdio>

int main() {
    int status;
    auto config = goud::EngineConfig::create(&status);
    std::printf("goud-engine test_package: config create status = %d\n", status);
    return 0;
}
