#nullable enable
using System;

namespace GoudEngine
{
    /// <summary>
    /// Handwritten convenience wrapper for network lifecycle operations.
    /// </summary>
    public sealed class NetworkManager
    {
        private readonly GoudContext? _context;
        private readonly GoudGame? _game;

        public NetworkManager(GoudGame game)
        {
            _game = game ?? throw new ArgumentNullException(nameof(game));
        }

        public NetworkManager(GoudContext context)
        {
            _context = context ?? throw new ArgumentNullException(nameof(context));
        }

        /// <summary>
        /// Hosts a network endpoint. Host endpoints do not have a default peer ID.
        /// </summary>
        public NetworkEndpoint Host(NetworkProtocol protocol, ushort port)
        {
            var handle = RunHost((int)protocol, port);
            return new NetworkEndpoint(_game, _context, handle, defaultPeerId: null);
        }

        /// <summary>
        /// Connects to a remote endpoint and preserves the provider-assigned default peer ID.
        /// </summary>
        public NetworkEndpoint Connect(NetworkProtocol protocol, string address, ushort port)
        {
            var result = RunConnectWithPeer((int)protocol, address, port);
            return new NetworkEndpoint(_game, _context, result.Handle, result.PeerId);
        }

        private long RunHost(int protocol, ushort port)
        {
            if (_game != null)
            {
                return _game.NetworkHost(protocol, port);
            }

            return _context!.NetworkHost(protocol, port);
        }

        private NetworkConnectResult RunConnectWithPeer(int protocol, string address, ushort port)
        {
            if (_game != null)
            {
                return _game.NetworkConnectWithPeer(protocol, address, port);
            }

            return _context!.NetworkConnectWithPeer(protocol, address, port);
        }
    }
}
