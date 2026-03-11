"""Entity wrapper generation."""

from sdk_common import write_generated
from .context import HEADER_COMMENT, NS, OUT

def gen_entity():
    lines = [
        f"// {HEADER_COMMENT}", f"namespace {NS}", "{",
        "    public struct Entity", "    {",
        "        private readonly ulong _bits;", "",
        "        public Entity(ulong bits) { _bits = bits; }", "",
        "        public uint Index => (uint)(_bits & 0xFFFFFFFF);",
        "        public uint Generation => (uint)(_bits >> 32);",
        "        public bool IsPlaceholder => _bits == ulong.MaxValue;",
        "        public ulong ToBits() => _bits;", "",
        "        public static readonly Entity Placeholder = new Entity(ulong.MaxValue);", "",
        '        public override string ToString() => $"Entity({Index}v{Generation})";',
        "    }", "}", "",
    ]
    write_generated(OUT / "Core" / "Entity.g.cs", "\n".join(lines))


# ── Component body generation for tool classes ───────────────────────
