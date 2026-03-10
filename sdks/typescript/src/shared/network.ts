import type {
  INetworkConnectResult,
  INetworkPacket,
  INetworkSimulationConfig,
  INetworkStats,
} from '../generated/types/engine.g.js';

export interface NetworkContextLike {
  networkHost(protocol: number, port: number): number;
  networkConnectWithPeer(protocol: number, address: string, port: number): INetworkConnectResult;
  networkReceivePacket(handle: number): INetworkPacket | null;
  networkSend(handle: number, peerId: number, data: Uint8Array, channel: number): number;
  networkPoll(handle: number): number;
  networkDisconnect(handle: number): number;
  getNetworkStats(handle: number): INetworkStats;
  networkPeerCount(handle: number): number;
  setNetworkSimulation(handle: number, config: INetworkSimulationConfig): number;
  clearNetworkSimulation(handle: number): number;
  setNetworkOverlayHandle(handle: number): number;
  clearNetworkOverlayHandle(): number;
}

export class NetworkManager {
  private readonly context: NetworkContextLike;

  constructor(gameOrContext: NetworkContextLike) {
    this.context = gameOrContext;
  }

  host(protocol: number, port: number): NetworkEndpoint {
    const handle = this.context.networkHost(protocol, port);
    if (handle < 0) {
      throw new Error(`networkHost failed with handle ${handle}`);
    }

    return new NetworkEndpoint(this.context, handle);
  }

  connect(protocol: number, address: string, port: number): NetworkEndpoint {
    const result = this.context.networkConnectWithPeer(protocol, address, port);
    return new NetworkEndpoint(this.context, result.handle, result.peerId);
  }
}

export class NetworkEndpoint {
  private readonly context: NetworkContextLike;

  readonly handle: number;

  readonly defaultPeerId: number | null;

  constructor(context: NetworkContextLike, handle: number, defaultPeerId: number | null = null) {
    this.context = context;
    this.handle = handle;
    this.defaultPeerId = defaultPeerId;
  }

  receive(): INetworkPacket | null {
    return this.context.networkReceivePacket(this.handle);
  }

  send(data: Uint8Array, channel = 0): number {
    if (this.defaultPeerId === null) {
      throw new Error('This endpoint has no default peer ID. Use sendTo(peerId, data, channel) instead.');
    }

    return this.sendTo(this.defaultPeerId, data, channel);
  }

  sendTo(peerId: number, data: Uint8Array, channel = 0): number {
    return this.context.networkSend(this.handle, peerId, data, channel);
  }

  poll(): number {
    return this.context.networkPoll(this.handle);
  }

  disconnect(): number {
    return this.context.networkDisconnect(this.handle);
  }

  getStats(): INetworkStats {
    return this.context.getNetworkStats(this.handle);
  }

  peerCount(): number {
    return this.context.networkPeerCount(this.handle);
  }

  setSimulation(config: INetworkSimulationConfig): number {
    return this.context.setNetworkSimulation(this.handle, config);
  }

  clearSimulation(): number {
    return this.context.clearNetworkSimulation(this.handle);
  }

  setOverlayTarget(): number {
    return this.context.setNetworkOverlayHandle(this.handle);
  }

  clearOverlayTarget(): number {
    return this.context.clearNetworkOverlayHandle();
  }
}
