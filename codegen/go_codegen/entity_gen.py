"""Generator for `goud/entity.go` -- EntityID type."""

from .context import GO_HEADER, OUT, schema, write_generated


def gen_entity() -> None:
    # Find the Entity handle type in schema
    entity_def = schema["types"].get("Entity")
    if not entity_def or entity_def.get("kind") != "handle":
        return

    lines = [
        GO_HEADER,
        "",
        "package goud",
        "",
        'import "fmt"',
        "",
        "// EntityID is a generational entity identifier.",
        "// The lower 32 bits are the index, the upper 32 bits are the generation.",
        "type EntityID uint64",
        "",
        "// NewEntityID creates an EntityID from raw bits.",
        "func NewEntityID(bits uint64) EntityID {",
        "\treturn EntityID(bits)",
        "}",
        "",
        "// Bits returns the raw uint64 representation.",
        "func (e EntityID) Bits() uint64 {",
        "\treturn uint64(e)",
        "}",
        "",
        "// Index returns the entity index (lower 32 bits).",
        "func (e EntityID) Index() uint32 {",
        "\treturn uint32(e & 0xFFFFFFFF)",
        "}",
        "",
        "// Generation returns the entity generation (upper 32 bits).",
        "func (e EntityID) Generation() uint32 {",
        "\treturn uint32(e >> 32)",
        "}",
        "",
        "// IsPlaceholder returns true if the entity is the placeholder sentinel.",
        "func (e EntityID) IsPlaceholder() bool {",
        "\treturn uint64(e) == 0xFFFFFFFFFFFFFFFF",
        "}",
        "",
        '// String implements fmt.Stringer.',
        "func (e EntityID) String() string {",
        '\treturn fmt.Sprintf("Entity(%dv%d)", e.Index(), e.Generation())',
        "}",
        "",
    ]

    write_generated(OUT / "entity.go", "\n".join(lines))
