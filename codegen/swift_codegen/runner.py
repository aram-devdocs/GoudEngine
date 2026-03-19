"""Entrypoint orchestration for Swift SDK generation."""

from .enums_gen import gen_enums
from .errors_gen import gen_errors
from .value_types_gen import gen_value_types
from .entity_gen import gen_entity
from .game_gen import gen_game
from .context_gen import gen_context
from .engine_config_gen import gen_engine_config
from .physics_gen import gen_physics
from .ui_gen import gen_ui
from .network_gen import gen_network


def run() -> None:
    print("Generating Swift SDK...")
    gen_enums()
    gen_errors()
    gen_value_types()
    gen_entity()
    gen_game()
    gen_context()
    gen_engine_config()
    gen_physics()
    gen_ui()
    gen_network()
    print("Swift SDK generation complete.")
