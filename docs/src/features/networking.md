# Networking

The networking SDK API is wrapper-based and sits on top of generated low-level bindings.

- `NetworkManager` creates endpoints from a game or context:
  - C#: `new NetworkManager(gameOrContext)`
  - Python: `NetworkManager(game_or_context)`
  - TypeScript: `new NetworkManager(gameOrContext)`
- `NetworkEndpoint` is returned by `host()` / `Host()` and `connect()` / `Connect()`. It exposes `receive`, `send`, `send_to` / `sendTo`, `poll`, `disconnect`, stats, peer count, simulation, and overlay helpers.

`connect()` stores the provider-assigned peer ID on the endpoint. Client code can call `send(...)` without passing a peer ID each time. Host endpoints do not have a default peer, so they reply with `send_to(...)` / `SendTo(...)`.

## CSharp

```csharp
using System.Text;
using GoudEngine;

using var hostContext = new GoudContext();
using var clientContext = new GoudContext();

var host = new NetworkManager(hostContext).Host(NetworkProtocol.Tcp, 9000);
var client = new NetworkManager(clientContext).Connect(NetworkProtocol.Tcp, "127.0.0.1", 9000);

client.Send(Encoding.UTF8.GetBytes("ping"));

while (true)
{
    host.Poll();
    client.Poll();

    var packet = host.Receive();
    if (packet is null)
    {
        continue;
    }

    host.SendTo(packet.Value.PeerId, Encoding.UTF8.GetBytes("pong"));
    break;
}
```

## Python

```python
from goud_engine import GoudContext, NetworkManager, NetworkProtocol

host_context = GoudContext()
client_context = GoudContext()

host = NetworkManager(host_context).host(NetworkProtocol.TCP, 9000)
client = NetworkManager(client_context).connect(NetworkProtocol.TCP, "127.0.0.1", 9000)

client.send(b"ping")

while True:
    host.poll()
    client.poll()

    packet = host.receive()
    if packet is None:
        continue

    host.send_to(packet.peer_id, b"pong")
    break
```

## TypeScript

```typescript
import { GoudContext, NetworkManager, NetworkProtocol } from "goudengine/node";

const hostContext = new GoudContext();
const clientContext = new GoudContext();

const host = new NetworkManager(hostContext).host(NetworkProtocol.Tcp, 9000);
const client = new NetworkManager(clientContext).connect(
  NetworkProtocol.Tcp,
  "127.0.0.1",
  9000,
);

client.send(Buffer.from("ping"));

while (true) {
  host.poll();
  client.poll();

  const packet = host.receive();
  if (!packet) {
    continue;
  }

  host.sendTo(packet.peerId, Buffer.from("pong"));
  break;
}
```

TypeScript networking is Node-only today (`goudengine/node`), including loopback/headless usage. `goudengine/web` exports the wrappers for type parity, but networking calls still throw `Not supported in WASM mode`.
