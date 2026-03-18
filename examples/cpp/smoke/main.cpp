#include <goud/goud.hpp>

int main() {
    int status = SUCCESS;
    auto config = goud::EngineConfig::createUnique(&status);

    if (status != SUCCESS || config == nullptr || !config->valid()) {
        return status;
    }

    status = config->setTitle("goud cpp smoke");
    if (status != SUCCESS) {
        return status;
    }

    auto context = goud::Context::create(&status);
    if (status != SUCCESS || !context.valid()) {
        return status;
    }

    std::uint64_t entity = GOUD_INVALID_ENTITY_ID;
    status = context.spawnEntity(entity);
    if (status != SUCCESS) {
        return status;
    }

    return context.reset();
}
