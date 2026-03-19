#include <catch2/catch_test_macros.hpp>
#include <goud/goud.hpp>

TEST_CASE("Context default is invalid", "[context]") {
    goud::Context ctx;
    REQUIRE_FALSE(ctx.valid());
}

TEST_CASE("Context move semantics", "[context]") {
    goud::Context a;
    goud::Context b(std::move(a));
    REQUIRE_FALSE(b.valid());
    REQUIRE_FALSE(a.valid());
}

// These tests require a GL context provided by the engine
TEST_CASE("Context::create with engine", "[context][gl_required]") {
    auto config = goud::EngineConfig::create();
    config.setTitle("test_context");
    config.setSize(64, 64);

    auto engine = goud::Engine::create(std::move(config));
    REQUIRE(engine.valid());

    auto& ctx = engine.context();
    REQUIRE(ctx.valid());

    std::uint64_t entity = GOUD_INVALID_ENTITY_ID;
    REQUIRE(ctx.spawnEntity(entity) == SUCCESS);
    REQUIRE(entity != GOUD_INVALID_ENTITY_ID);
    REQUIRE(ctx.isEntityAlive(entity));

    REQUIRE(ctx.destroyEntity(entity) == SUCCESS);
    REQUIRE_FALSE(ctx.isEntityAlive(entity));
}
