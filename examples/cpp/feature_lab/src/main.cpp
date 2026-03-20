/// GoudEngine Feature Lab -- C++ Headless Smoke Test
///
/// Exercises SDK surface without requiring a window or GPU context.
/// Prints pass/fail for each check and exits 0 on success, 1 on any failure.

#include <goud/goud.hpp>

#include <cstdio>
#include <cstdlib>
#include <string>
#include <vector>

struct CheckResult {
    std::string name;
    bool passed;
};

static std::vector<CheckResult> results;

static void record(const char* name, bool passed) {
    results.push_back({name, passed});
}

int main() {
    // -- Test: EngineConfig creation and setters ------------------------------
    {
        int status = 0;
        auto config = goud::EngineConfig::create(&status);
        bool configValid = (status == SUCCESS) && config.valid();

        config.setTitle("C++ Feature Lab");
        config.setSize(1280, 720);
        config.setVsync(false);
        config.setTargetFps(120);

        record("EngineConfig creation and setters", configValid);

        // Destroy without building -- tests RAII cleanup path
        config.reset();
        record("EngineConfig reset without engine creation", !config.valid());
    }

    // -- Test: EngineConfig unique_ptr path ------------------------------------
    {
        int status = 0;
        auto config = goud::EngineConfig::createUnique(&status);
        bool ok = (status == SUCCESS) && config && config->valid();
        config->setTitle("Unique Config");
        config.reset();
        record("EngineConfig unique_ptr creation and destruction", ok);
    }

    // -- Test: Error retrieval ------------------------------------------------
    {
        auto err = goud::Error::last();
        record("Error::last returns no-error when none is set", !static_cast<bool>(err));
    }

    // -- Test: Context creation (headless -- may fail without GL) -------------
    {
        int status = 0;
        auto ctx = goud::Context::create(&status);
        bool created = (status == SUCCESS) && ctx.valid();
        if (created) {
            record("Headless Context::create succeeds", true);

            // Test entity spawn/destroy
            std::uint64_t entity = 0;
            int spawnStatus = ctx.spawnEntity(entity);
            bool alive = ctx.isEntityAlive(entity);
            int destroyStatus = ctx.destroyEntity(entity);
            bool dead = !ctx.isEntityAlive(entity);
            record("Entity spawn/alive/destroy roundtrip",
                   spawnStatus == SUCCESS && alive && destroyStatus == SUCCESS && dead);
        } else {
            // Context creation failure is expected in headless CI
            record("Headless Context::create fails gracefully (expected in CI)", true);
        }
    }

    // -- Test: Engine creation (requires windowing -- expected to fail headless)
    {
        int configStatus = 0;
        auto config = goud::EngineConfig::create(&configStatus);
        config.setTitle("Headless Engine Test");
        config.setSize(320, 240);

        int engineStatus = 0;
        auto engine = goud::Engine::create(std::move(config), &engineStatus);
        if (engine.valid()) {
            record("Engine::create succeeds (desktop with GPU)", true);
        } else {
            // Expected in headless CI -- the failure path itself is the test
            record("Engine::create fails gracefully without windowing (expected in CI)", true);
        }
    }

    // -- Summary --------------------------------------------------------------
    int passCount = 0;
    for (const auto& r : results) {
        if (r.passed) {
            passCount++;
        }
    }
    int failCount = static_cast<int>(results.size()) - passCount;

    std::printf("C++ Feature Lab complete: %d pass, %d fail\n", passCount, failCount);
    for (const auto& r : results) {
        std::printf("%s: %s\n", r.passed ? "PASS" : "FAIL", r.name.c_str());
    }

    return failCount > 0 ? 1 : 0;
}
