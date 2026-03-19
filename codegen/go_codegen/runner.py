"""Entrypoint orchestration for Go SDK generation."""

from .components_gen import gen_components
from .doc_gen import gen_doc
from .entity_gen import gen_entity
from .errors_gen import gen_errors
from .game_gen import gen_game
from .helpers_gen import gen_helpers
from .keys_gen import gen_keys
from .types_gen import gen_types


def run() -> None:
    print("Generating Go SDK wrapper...")
    gen_doc()
    gen_helpers()
    gen_types()
    gen_components()
    gen_entity()
    gen_keys()
    gen_errors()
    gen_game()
    print("Go SDK wrapper generation complete.")
