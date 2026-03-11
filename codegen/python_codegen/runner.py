"""Entrypoint orchestration for Python SDK generation."""

from .diagnostic_gen import gen_diagnostic
from .errors_gen import gen_errors
from .ffi_gen import gen_ffi
from .game_gen import gen_game
from .init_gen import gen_init
from .keys_gen import gen_keys
from .networking_gen import gen_network_wrappers
from .types_gen import gen_types


def run() -> None:
    print("Generating Python SDK...")
    gen_ffi()
    gen_keys()
    gen_types()
    gen_game()
    gen_network_wrappers()
    gen_errors()
    gen_diagnostic()
    gen_init()
    print("Python SDK generation complete.")
