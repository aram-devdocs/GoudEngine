"""C# SDK generator package."""

from .native_enums import gen_native_methods, gen_enums
from .value_types import gen_value_types
from .components import gen_component_wrappers
from .entity import gen_entity
from .tools import gen_game, gen_context
from .physics_ui import (
    gen_physics_world_2d,
    gen_physics_world_3d,
    gen_engine_config,
    gen_ui_manager,
)
from .errors_network import gen_errors, gen_network_wrappers
from .diagnostic import gen_diagnostic


def generate_all() -> None:
    gen_native_methods()
    gen_enums()
    gen_value_types()
    gen_component_wrappers()
    gen_entity()
    gen_game()
    gen_context()
    gen_physics_world_2d()
    gen_physics_world_3d()
    gen_engine_config()
    gen_ui_manager()
    gen_network_wrappers()
    gen_errors()
    gen_diagnostic()
