"""Shared context/constants for C# code generation."""

from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from sdk_common import (
    HEADER_COMMENT,
    SDKS_DIR,
    load_schema,
    load_ffi_mapping,
)

OUT = SDKS_DIR / "csharp" / "generated"
CS_ROOT = SDKS_DIR / "csharp"
schema = load_schema()
mapping = load_ffi_mapping(schema)
NS = "GoudEngine"

# FFI struct field names (PascalCase as declared in codegen NativeMethods)
_FFI_TO_SDK_FIELDS = {
    "FfiColor": ["R", "G", "B", "A"],
    "FfiVec2": ["X", "Y"],
    "FfiVec3": ["X", "Y", "Z"],
    "FfiRect": ["X", "Y", "Width", "Height"],
    "FfiTransform2D": ["PositionX", "PositionY", "Rotation", "ScaleX", "ScaleY"],
}
_FFI_TO_SDK_RETURN = {
    "FfiTransform2D": "Transform2D",
    "FfiSprite": "Sprite",
    "FfiColor": "Color",
    "FfiVec2": "Vec2",
    "FfiVec3": "Vec3",
    "FfiRect": "Rect",
    "FfiMat3x3": "Mat3x3",
}

_CSHARP_KEYWORDS = {
    "abstract", "as", "base", "bool", "break", "byte", "case", "catch", "char", "checked",
    "class", "const", "continue", "decimal", "default", "delegate", "do", "double", "else",
    "enum", "event", "explicit", "extern", "false", "finally", "fixed", "float", "for",
    "foreach", "goto", "if", "implicit", "in", "int", "interface", "internal", "is", "lock",
    "long", "namespace", "new", "null", "object", "operator", "out", "override", "params",
    "private", "protected", "public", "readonly", "ref", "return", "sbyte", "sealed", "short",
    "sizeof", "stackalloc", "static", "string", "struct", "switch", "this", "throw", "true",
    "try", "typeof", "uint", "ulong", "unchecked", "unsafe", "ushort", "using", "virtual",
    "void", "volatile", "while",
}

_CSHARP_FFI_ALIASES = {
    "EngineConfigHandle": "*mut c_void",
    "UiManagerHandle": "*mut c_void",
    "GoudTextureHandle": "u64",
    "GoudFontHandle": "u64",
    "GoudEntityId": "u64",
    "GoudKeyCode": "i32",
    "GoudMouseButton": "i32",
    "GoudErrorCode": "i32",
}
