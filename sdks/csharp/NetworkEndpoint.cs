#nullable enable
using System;

namespace GoudEngine
{
    /// <summary>
    /// Handwritten convenience wrapper for a concrete network handle.
    /// </summary>
    public sealed class NetworkEndpoint
    {
        private readonly GoudContext? _context;
        private readonly GoudGame? _game;

        internal NetworkEndpoint(GoudGame? game, GoudContext? context, long handle, ulong? defaultPeerId)
        {
            _game = game;
            _context = context;
            Handle = handle;
            DefaultPeerId = defaultPeerId;
        }

        /// <summary>
        /// Underlying native network handle.
        /// </summary>
        public long Handle { get; }

        /// <summary>
        /// Default peer ID used by <see cref="Send(byte[], byte)"/>, when present.
        /// </summary>
        public ulong? DefaultPeerId { get; }

        /// <summary>
        /// Sends bytes to the endpoint's default peer on the given channel.
        /// </summary>
        public int Send(byte[] data, byte channel = 0)
        {
            if (!DefaultPeerId.HasValue)
            {
                throw new InvalidOperationException(
                    "No default peer ID is available for this endpoint. Use SendTo(...) or connect via NetworkManager.Connect(...)."
                );
            }

            return SendTo(DefaultPeerId.Value, data, channel);
        }

        /// <summary>
        /// Sends bytes to a specific peer on the given channel.
        /// </summary>
        public int SendTo(ulong peerId, byte[] data, byte channel = 0)
        {
            if (_game != null)
            {
                return _game.NetworkSend(Handle, peerId, data, channel);
            }

            return _context!.NetworkSend(Handle, peerId, data, channel);
        }

        /// <summary>
        /// Polls the provider and buffers pending events/messages for this handle.
        /// </summary>
        public int Poll()
        {
            if (_game != null)
            {
                return _game.NetworkPoll(Handle);
            }

            return _context!.NetworkPoll(Handle);
        }

        /// <summary>
        /// Receives the next buffered packet or null when no packet is available.
        /// </summary>
        public NetworkPacket? Receive()
        {
            if (_game != null)
            {
                return _game.NetworkReceivePacket(Handle);
            }

            return _context!.NetworkReceivePacket(Handle);
        }

        /// <summary>
        /// Disconnects this endpoint handle.
        /// </summary>
        public int Disconnect()
        {
            if (_game != null)
            {
                return _game.NetworkDisconnect(Handle);
            }

            return _context!.NetworkDisconnect(Handle);
        }

        /// <summary>
        /// Returns aggregate stats for this endpoint handle.
        /// </summary>
        public NetworkStats GetStats()
        {
            if (_game != null)
            {
                return _game.GetNetworkStats(Handle);
            }

            return _context!.GetNetworkStats(Handle);
        }

        /// <summary>
        /// Returns the number of peers associated with this endpoint handle.
        /// </summary>
        public int PeerCount()
        {
            if (_game != null)
            {
                return _game.NetworkPeerCount(Handle);
            }

            return _context!.NetworkPeerCount(Handle);
        }

        /// <summary>
        /// Applies debug network simulation config to this endpoint handle.
        /// </summary>
        public int SetSimulation(NetworkSimulationConfig config)
        {
            if (_game != null)
            {
                return _game.SetNetworkSimulation(Handle, config);
            }

            return _context!.SetNetworkSimulation(Handle, config);
        }

        /// <summary>
        /// Clears debug network simulation config from this endpoint handle.
        /// </summary>
        public int ClearSimulation()
        {
            if (_game != null)
            {
                return _game.ClearNetworkSimulation(Handle);
            }

            return _context!.ClearNetworkSimulation(Handle);
        }

        /// <summary>
        /// Routes native network overlay stats to this endpoint handle.
        /// </summary>
        public int SetOverlayTarget()
        {
            if (_game != null)
            {
                return _game.SetNetworkOverlayHandle(Handle);
            }

            return _context!.SetNetworkOverlayHandle(Handle);
        }

        /// <summary>
        /// Clears the native network overlay target override for this context.
        /// </summary>
        public int ClearOverlayTarget()
        {
            if (_game != null)
            {
                return _game.ClearNetworkOverlayHandle();
            }

            return _context!.ClearNetworkOverlayHandle();
        }
    }
}
