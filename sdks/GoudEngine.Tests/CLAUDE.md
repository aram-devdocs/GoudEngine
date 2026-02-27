# GoudEngine.Tests/ — C# SDK Tests

## Purpose

xUnit test suite for the C# SDK.

## Structure

- `Components/ComponentTests.cs` — Component wrapper tests
- `Core/EntityTests.cs` — Entity creation and management tests
- `Core/ExceptionTests.cs` — Exception mapping tests
- `Core/IdentifierTests.cs` — ID generation and uniqueness tests
- `Math/ColorTests.cs` — Color type tests
- `Math/RectangleTests.cs` — Rectangle type tests
- `Math/Vector2Tests.cs` — Vector2 math tests
- `Math/Vector3Tests.cs` — Vector3 math tests

## Running

```bash
dotnet test sdks/GoudEngine.Tests/
dotnet test sdks/GoudEngine.Tests/ --filter "FullyQualifiedName~Math"  # Math only
```

## Patterns

- Math tests are pure computation — no GL context needed
- Component/Entity tests may need native library loaded
- Use xUnit `[Fact]` for single cases, `[Theory]` for parameterized
- Arrange-Act-Assert pattern for all tests
- One assertion per concept

## Anti-Patterns

- NEVER skip tests with `[Fact(Skip = "...")]` without a tracking issue
- NEVER write tests without assertions
- NEVER test Rust logic from C# — test the SDK wrapper behavior only
