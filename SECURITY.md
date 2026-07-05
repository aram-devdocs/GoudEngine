# Security Policy

## Supported Versions

The project is in `0.0.x` alpha. Only the latest published `0.0.x` release
receives security fixes. Older releases are not patched; upgrade to the latest
version before reporting an issue against an older one.

| Version        | Supported          |
| -------------- | ------------------ |
| Latest `0.0.x` | :white_check_mark: |
| Any older      | :x:                |

## Reporting a Vulnerability

Report vulnerabilities through GitHub's private security advisory flow, not a
public issue:

<https://github.com/aram-devdocs/GoudEngine/security/advisories/new>

The advisory keeps the report private between reporter and maintainers until a
fix ships. Include the affected version, a reproduction, and the impact you
observed. You SHOULD receive an acknowledgement within a few days.

## Coordinated Disclosure

The project follows coordinated disclosure with a 90-day window. Maintainers
work to release a fix and publish the advisory before the window closes.
Reporters SHOULD keep details private until the advisory is published or the
window elapses, whichever comes first. When a fix lands sooner, the advisory
MAY be published early by mutual agreement.

## Scope

The following boundaries are the primary security surface. Reports against them
are in scope:

- **FFI boundary** (`goud_engine/src/ffi/`). Every SDK reaches the engine
  through C-ABI functions. Memory-safety defects at this boundary (missing null
  checks, unsound `unsafe`, ownership or lifetime violations across the
  boundary) are in scope.
- **Codegen pipeline** (`codegen/`). The generator turns the FFI surface into
  SDK bindings. Defects that emit unsound or unsafe bindings for any target
  language are in scope.
- **MCP debug server** (`tools/goudengine-mcp/`). The debug server and its
  WebSocket relay bind to `127.0.0.1` (localhost) only and are intended for
  local development. A defect that causes it to accept non-local connections,
  or that lets a local client escalate beyond the documented debug surface, is
  in scope.

Out of scope: vulnerabilities in third-party dependencies (report those
upstream), and issues that require an already-compromised host.
