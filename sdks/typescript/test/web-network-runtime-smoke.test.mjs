import { describe, it } from 'node:test';

import { runWebNetworkingRuntimeSmoke } from './web-runtime-smoke-lib.mjs';

describe('web networking runtime smoke (browser + wasm)', () => {
  it(
    'connects to a live websocket host and round-trips one payload',
    async () => {
      await runWebNetworkingRuntimeSmoke();
    },
    { timeout: 120_000 },
  );
});
