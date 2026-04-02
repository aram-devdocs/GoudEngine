using System;
using System.Runtime.InteropServices;
using System.Text;
using GoudEngine;
using Xunit;
using Xunit.Abstractions;

namespace GoudEngine.Tests.Network;

/// <summary>
/// Integration tests validating C# RPC SDK for Throne server/client architecture.
///
/// The RPC framework is transport-agnostic: it produces/consumes byte buffers that must be
/// shuttled over a network transport (TCP/UDP). These tests simulate the transport by manually
/// calling DrainOne and ProcessIncoming.
///
/// Observed characteristics:
/// - RPC echo round-trip completes in a single poll cycle when using in-process byte shuttling.
/// - The default FFI handler echoes the call payload back (see ffi/network/rpc.rs:165).
///
/// Known limitations:
/// - goud_rpc_drain_one (rpc.rs:392-397) drains only one outbound message per call. If multiple
///   messages queue between drains, only the first is returned and the rest are lost. This does
///   not affect the single-call echo test but would break batch RPC scenarios.
/// - C# SDK lacks RpcCall, RpcReceiveResponse, and RpcDrainOne wrapper methods — these tests
///   call NativeMethods directly via RpcTestHelper. SDK parity gap tracked separately.
/// </summary>
public class RpcIntegrationTests
{
    private readonly ITestOutputHelper _output;

    public RpcIntegrationTests(ITestOutputHelper output)
    {
        _output = output;
    }

    [Fact]
    public void Rpc_Register_Call_Echo_Round_Trip()
    {
        long serverHandle = -1;
        long clientHandle = -1;

        try
        {
            // Create two RPC framework instances
            serverHandle = RpcTestHelper.Create(5000, 4096);
            Assert.True(serverHandle >= 0, $"server RPC create failed with handle {serverHandle}");

            clientHandle = RpcTestHelper.Create(5000, 4096);
            Assert.True(clientHandle >= 0, $"client RPC create failed with handle {clientHandle}");

            // Register RPC ID 1 "ping" as Bidirectional on both
            const ushort rpcId = 1;
            const string rpcName = "ping";

            Assert.Equal(0, RpcTestHelper.Register(serverHandle, rpcId, rpcName, RpcDirection.Bidirectional));
            Assert.Equal(0, RpcTestHelper.Register(clientHandle, rpcId, rpcName, RpcDirection.Bidirectional));

            // Client calls the RPC
            var payload = Encoding.UTF8.GetBytes("hello-rpc");
            ulong callId = RpcTestHelper.Call(clientHandle, peerId: 1, rpcId, payload);
            _output.WriteLine($"Client call ID: {callId}");

            // Drain client outbox
            var (drainedData, drainedPeerId) = RpcTestHelper.DrainOne(clientHandle, 4096);
            Assert.NotNull(drainedData);
            Assert.True(drainedData!.Length > 0, "drained data should not be empty");
            _output.WriteLine($"Drained {drainedData.Length} bytes for peer {drainedPeerId}");

            // Feed to server as incoming from peer 1
            Assert.Equal(0, RpcTestHelper.ProcessIncoming(serverHandle, peerId: 1, drainedData));

            // Poll server to process the incoming call
            Assert.Equal(0, RpcTestHelper.Poll(serverHandle, 0.016f));

            // Drain server outbox -- should contain the echo response
            var (serverResponse, serverResponsePeerId) = RpcTestHelper.DrainOne(serverHandle, 4096);
            Assert.NotNull(serverResponse);
            Assert.True(serverResponse!.Length > 0, "server response should not be empty");
            _output.WriteLine($"Server response: {serverResponse.Length} bytes for peer {serverResponsePeerId}");

            // Feed server response back to client as incoming from peer 1
            Assert.Equal(0, RpcTestHelper.ProcessIncoming(clientHandle, peerId: 1, serverResponse));

            // Poll client to process the response
            Assert.Equal(0, RpcTestHelper.Poll(clientHandle, 0.016f));

            // Receive the response on the client side
            var responseData = RpcTestHelper.ReceiveResponse(clientHandle, callId, 4096);
            Assert.NotNull(responseData);
            Assert.True(responseData!.Length > 0, "response payload should not be empty");

            // Verify the echoed payload matches
            var responseText = Encoding.UTF8.GetString(responseData);
            _output.WriteLine($"Response text: {responseText}");
            Assert.Equal("hello-rpc", responseText);
        }
        finally
        {
            if (clientHandle >= 0)
            {
                RpcTestHelper.Destroy(clientHandle);
            }

            if (serverHandle >= 0)
            {
                RpcTestHelper.Destroy(serverHandle);
            }
        }
    }

    private static class RpcTestHelper
    {
        public static long Create(ulong timeoutMs, uint maxPayload)
        {
            return NativeMethods.goud_rpc_create(timeoutMs, maxPayload);
        }

        public static void Destroy(long handle)
        {
            NativeMethods.goud_rpc_destroy(handle);
        }

        public static int Register(long handle, ushort rpcId, string name, RpcDirection direction)
        {
            var nameBytes = Encoding.UTF8.GetBytes(name);
            var pin = GCHandle.Alloc(nameBytes, GCHandleType.Pinned);
            try
            {
                return NativeMethods.goud_rpc_register(
                    handle,
                    rpcId,
                    pin.AddrOfPinnedObject(),
                    nameBytes.Length,
                    (int)direction
                );
            }
            finally
            {
                pin.Free();
            }
        }

        public static ulong Call(long handle, ulong peerId, ushort rpcId, byte[] payload)
        {
            ulong callIdOut = 0;
            var pin = GCHandle.Alloc(payload, GCHandleType.Pinned);
            try
            {
                var result = NativeMethods.goud_rpc_call(
                    handle,
                    peerId,
                    rpcId,
                    pin.AddrOfPinnedObject(),
                    payload.Length,
                    ref callIdOut
                );
                Assert.Equal(0, result);
            }
            finally
            {
                pin.Free();
            }

            return callIdOut;
        }

        public static int Poll(long handle, float deltaSecs)
        {
            return NativeMethods.goud_rpc_poll(handle, deltaSecs);
        }

        public static int ProcessIncoming(long handle, ulong peerId, byte[] data)
        {
            var pin = GCHandle.Alloc(data, GCHandleType.Pinned);
            try
            {
                return NativeMethods.goud_rpc_process_incoming(
                    handle,
                    peerId,
                    pin.AddrOfPinnedObject(),
                    data.Length
                );
            }
            finally
            {
                pin.Free();
            }
        }

        public static (byte[]? data, ulong peerId) DrainOne(long handle, int bufferSize)
        {
            var buffer = new byte[bufferSize];
            ulong outPeerId = 0;
            var pin = GCHandle.Alloc(buffer, GCHandleType.Pinned);
            int bytesWritten;
            try
            {
                bytesWritten = NativeMethods.goud_rpc_drain_one(
                    handle,
                    pin.AddrOfPinnedObject(),
                    bufferSize,
                    ref outPeerId
                );
            }
            finally
            {
                pin.Free();
            }

            if (bytesWritten <= 0)
            {
                return (null, 0);
            }

            var result = new byte[bytesWritten];
            Array.Copy(buffer, result, bytesWritten);
            return (result, outPeerId);
        }

        public static byte[]? ReceiveResponse(long handle, ulong callId, int bufferSize)
        {
            var buffer = new byte[bufferSize];
            int outWritten = 0;
            var pin = GCHandle.Alloc(buffer, GCHandleType.Pinned);
            int result;
            try
            {
                result = NativeMethods.goud_rpc_receive_response(
                    handle,
                    callId,
                    pin.AddrOfPinnedObject(),
                    bufferSize,
                    ref outWritten
                );
            }
            finally
            {
                pin.Free();
            }

            if (result != 0 || outWritten <= 0)
            {
                return null;
            }

            var responseData = new byte[outWritten];
            Array.Copy(buffer, responseData, outWritten);
            return responseData;
        }
    }
}
