# Networking

GoudEngine provides a client-server networking foundation with server-authoritative state flow.

## Core Model

The networking module centers on two types:

- `SessionServer` for hosting, accepting clients, validating commands, and broadcasting authoritative updates
- `SessionClient` for joining sessions, receiving authoritative state, and handling leave/disconnect lifecycle

## Authority

Server authority is pluggable.

- Built-in policies include allow-all, schema+bounds validation, and rate-limited validation
- Commands are validated on the server before state updates are accepted
- Rejected commands are surfaced as explicit events

## Discovery And Join

Clients can join with direct addresses or discovered sessions.

- Direct join path: explicit address
- Discovery path: local network and directory-backed discovery modes
- Join/leave lifecycle is designed for mid-session churn

## Protocol

The protocol covers:

- Join request/accept handshake
- Client state-change commands
- Server authoritative state update broadcasts
- Rejection responses and graceful leave notices

## Testing Focus

The networking test suite covers:

- Authority validation and rejection behavior
- Multi-client host/join and broadcast flow
- Discovery modes and rehost behavior
- Protocol source validation and lifecycle edge cases
