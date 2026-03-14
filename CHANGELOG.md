# Changelog

## [0.0.831](https://github.com/aram-devdocs/GoudEngine/compare/v0.0.830...v0.0.831) (2026-03-14)


### Bug Fixes

* resolve Windows CI test flakes ([813d15b](https://github.com/aram-devdocs/GoudEngine/commit/813d15b8f9a2a977bdbe5e058458f54015593db2))
* resolve Windows CI test flakes in texture and debugger tests ([4c3a3d0](https://github.com/aram-devdocs/GoudEngine/commit/4c3a3d03e0bd58fd1222bc1a2cecdde2145047b9))

## [0.0.830](https://github.com/aram-devdocs/GoudEngine/compare/v0.0.829...v0.0.830) (2026-03-14)


### Bug Fixes

* remove version field from ffi_manifest.json to prevent release-please drift ([#540](https://github.com/aram-devdocs/GoudEngine/issues/540)) ([8e8e3ad](https://github.com/aram-devdocs/GoudEngine/commit/8e8e3ad0b1b5919ea3b188b86f4bbc6d2610c1ba))

## [0.0.829](https://github.com/aram-devdocs/GoudEngine/compare/v0.0.828...v0.0.829) (2026-03-13)


### Features

* 100% MCP tool pass rate on Rust, C#, Python, and TS Desktop ([df7541e](https://github.com/aram-devdocs/GoudEngine/commit/df7541e0ceea5af70b7278163d7c219498097272))
* add animation blending, layering, and events ([0e619cc](https://github.com/aram-devdocs/GoudEngine/commit/0e619cc9087e4e3f9d06a37d45e96056abc80a03))
* add animation blending, layering, and events ([#176](https://github.com/aram-devdocs/GoudEngine/issues/176), [#184](https://github.com/aram-devdocs/GoudEngine/issues/184)) ([7db89be](https://github.com/aram-devdocs/GoudEngine/commit/7db89be9a2cdaf52654a35a426b9ac79031c88ce))
* add asset packaging for distribution with archive VFS backend ([5336781](https://github.com/aram-devdocs/GoudEngine/commit/533678196a74b9d8091b1bf189656d8fcee0f90a))
* add asset pipeline features (VFS, packaging, compressed textures, ref counting, fallbacks) ([0b6a123](https://github.com/aram-devdocs/GoudEngine/commit/0b6a123c751c2fe9551042d20e78d1fbbc3a324b))
* add configurable physics backends and debug overlay ([20af035](https://github.com/aram-devdocs/GoudEngine/commit/20af0352ffd7034f4b15e2132b92e3826ecc543f))
* add configurable physics backends and debug overlay ([d6a908d](https://github.com/aram-devdocs/GoudEngine/commit/d6a908dcaa5599171cb0f3f2011a37bb3d5d57a2))
* add core client-server networking architecture ([#482](https://github.com/aram-devdocs/GoudEngine/issues/482)) ([8348f63](https://github.com/aram-devdocs/GoudEngine/commit/8348f63f557935f5d348d7425d86b38f9cbade25))
* add DDS container parsing and compressed texture support (BC1/BC3/BC5/BC7) ([2f0f533](https://github.com/aram-devdocs/GoudEngine/commit/2f0f533ffe2671a55623930cbb8383f9e128cab0))
* add debugger attach and control substrate ([5847eac](https://github.com/aram-devdocs/GoudEngine/commit/5847eace7f60680048651a881ececabf3be2176b))
* add debugger attach transport ([8d9c8f0](https://github.com/aram-devdocs/GoudEngine/commit/8d9c8f029551d918abb7184d45673e7de3e67e27))
* add debugger control, capture, replay, metrics, and MCP bridge ([4b2ba3d](https://github.com/aram-devdocs/GoudEngine/commit/4b2ba3d6abbc3243286fe7aba9f79c2605c2fd58))
* add debugger debug draw store ([069211e](https://github.com/aram-devdocs/GoudEngine/commit/069211e730be5758abc018dcb5fdc6c5c494cc7b))
* add debugger ffi control exports ([7220df7](https://github.com/aram-devdocs/GoudEngine/commit/7220df7bd12dab15ab861f697da0939475341a63))
* add debugger frame capture artifacts ([adf06c6](https://github.com/aram-devdocs/GoudEngine/commit/adf06c6dee07d88ed4ee2ce662495d43f5329514))
* add debugger metrics trace export ([2987d52](https://github.com/aram-devdocs/GoudEngine/commit/2987d52122e97d8051edaeb094c8fe782d64351a))
* add debugger replay recording pipeline ([95c2da1](https://github.com/aram-devdocs/GoudEngine/commit/95c2da198730f471423ecc0f07c19b5edf60c039))
* add debugger runtime adapters ([d3a51b0](https://github.com/aram-devdocs/GoudEngine/commit/d3a51b02be6388ade4f6bf17516868a2626e62c2))
* add debugger runtime substrate ([baaaaf7](https://github.com/aram-devdocs/GoudEngine/commit/baaaaf75f94768b0f59c7e38ce1641b1311b22ee))
* add debugger runtime, profiler, stats, and inspector ([e886f06](https://github.com/aram-devdocs/GoudEngine/commit/e886f06f93ac666b69209dca9be6f4fafefcc092))
* add debugger sdk and mcp bridge rollout ([fa69363](https://github.com/aram-devdocs/GoudEngine/commit/fa69363235a6a849c66bce2ae19951e84891f01d))
* add DeltaEncode impl for Transform2D in ecs layer ([cabbf46](https://github.com/aram-devdocs/GoudEngine/commit/cabbf4669f2522990bc5977d5ec022639aa00811))
* add dev-mode keyboard shortcut (F5) to cycle render provider ([a3bc89f](https://github.com/aram-devdocs/GoudEngine/commit/a3bc89f02ad1f0403027f3549cd2f0e2a82f0781))
* add diagnostic section to codegen generators ([666f8d5](https://github.com/aram-devdocs/GoudEngine/commit/666f8d5fa57bca638576b9486fa937aed1b2d979))
* add diagnostics timeline recording with time-slicing ([2463f60](https://github.com/aram-devdocs/GoudEngine/commit/2463f60733a76ff406c0972fc4945874a1a84226))
* add error logging integration and debug/diagnostic mode ([f8778e7](https://github.com/aram-devdocs/GoudEngine/commit/f8778e701495cf39f1d75f9bab614417245c8f22))
* add error logging integration and debug/diagnostic mode ([d56f2bd](https://github.com/aram-devdocs/GoudEngine/commit/d56f2bd4d5e8731cb61cb55e32c4ab589ba366f1))
* add fallback/default assets on load failure ([42889fd](https://github.com/aram-devdocs/GoudEngine/commit/42889fdd52c7ae63e1063790dd0e6ba87a4e9b57))
* add FFI exports for gravity, collider material properties, and codegen ([b7bbc93](https://github.com/aram-devdocs/GoudEngine/commit/b7bbc934b1a5e1a243dc37d7d8944c1389d7a01f))
* add fixed timestep integration, gravity scale, and collider material APIs ([7072c53](https://github.com/aram-devdocs/GoudEngine/commit/7072c5302c0d090f8c04ee412a59ca5eb3d42f23))
* add framebuffer readback seam ([a5abbf8](https://github.com/aram-devdocs/GoudEngine/commit/a5abbf823324a5db5a115d6a207bbe85d5f8a840))
* add lobby matchmaking flow ([c6f895b](https://github.com/aram-devdocs/GoudEngine/commit/c6f895bdb36857643029a309caa146f955b806b4))
* add minimal state sync slice ([9736bfc](https://github.com/aram-devdocs/GoudEngine/commit/9736bfc6d50bc34ae8aff6496048256f52c43240))
* add minimal state sync wrapper layer ([98c2649](https://github.com/aram-devdocs/GoudEngine/commit/98c264972fee24e5318f73a601342ed1e1229759))
* add native tcp transport and debug network sim ([58d7786](https://github.com/aram-devdocs/GoudEngine/commit/58d7786e3aed526c5c01634ef9e0ee8999a439c3))
* add network simulation, overlay, and SDK access ([b3a77a7](https://github.com/aram-devdocs/GoudEngine/commit/b3a77a7a00cafae3d7ef07a5c4c7a2a58f402538))
* add networking sdk wrappers ([a3cd28c](https://github.com/aram-devdocs/GoudEngine/commit/a3cd28cbb6852a1a6dc0e3569d4f8ef6452619ff))
* add provider hot-swap and capability query API ([e804418](https://github.com/aram-devdocs/GoudEngine/commit/e8044184bbe60d457a8d1fb05c1d6d77b42f51a9))
* add provider hot-swap and capability query API ([7a8417f](https://github.com/aram-devdocs/GoudEngine/commit/7a8417f680f9852d5567eeced606f5677605b138)), closes [#249](https://github.com/aram-devdocs/GoudEngine/issues/249) [#252](https://github.com/aram-devdocs/GoudEngine/issues/252)
* add reference counting for asset handles with deferred unload ([47e14f3](https://github.com/aram-devdocs/GoudEngine/commit/47e14f39612a419adf2c4b57af8a7fb06252bf90))
* add renderer3d debug draw pass ([5baf1d5](https://github.com/aram-devdocs/GoudEngine/commit/5baf1d5f0ea3a014a9aae137893fb539572d66a5))
* add sandbox parity app ([f1c3bb4](https://github.com/aram-devdocs/GoudEngine/commit/f1c3bb40dfe13308dcb68df0171d66194b08a14b))
* add scene transition FFI exports and codegen schema ([fd0778e](https://github.com/aram-devdocs/GoudEngine/commit/fd0778e78855a01a9c91c66bbf98a1031b590ed3))
* add scene transition state machine ([e50b0a1](https://github.com/aram-devdocs/GoudEngine/commit/e50b0a1f80eecd2771f6b15f1d00bf41c7693993))
* add scene transitions (fade, instant, custom) ([f7397b3](https://github.com/aram-devdocs/GoudEngine/commit/f7397b30050115ec3ec4292f5058bcf9c909af14))
* add serialization framework with binary encoding and delta compression ([25b3938](https://github.com/aram-devdocs/GoudEngine/commit/25b39381235d2e3e8296c4524d4abbd378a00fe3))
* add serialization framework with binary encoding and delta compression ([c69592b](https://github.com/aram-devdocs/GoudEngine/commit/c69592b8a0fd1d8f64a16c7a9658b319431d7269))
* add standalone ui manager ffi and sdk parity ([7ef40fe](https://github.com/aram-devdocs/GoudEngine/commit/7ef40fe2a1f798fb2cce1c8af47922956f2c4582))
* add standalone ui manager ffi and sdk parity ([5d10d69](https://github.com/aram-devdocs/GoudEngine/commit/5d10d69b25304a6608dec5b6964c6d38fe6cf039))
* add state sync and lobby matchmaking ([43e1f5c](https://github.com/aram-devdocs/GoudEngine/commit/43e1f5c24d204fe90748fd45cb572d8d1d17a6a6))
* add text batch renderer, text render system, and bitmap atlas adapter ([0b3ccee](https://github.com/aram-devdocs/GoudEngine/commit/0b3ccee73950b9aed934b1dcb3db605fd7b04916))
* add text component FFI exports and SDK codegen ([4dd38a2](https://github.com/aram-devdocs/GoudEngine/commit/4dd38a2d97e227d03ebb26c41d3b2d71e26ecb4e))
* add text layout engine, text component, and BMFont parser ([656e289](https://github.com/aram-devdocs/GoudEngine/commit/656e28936d5846aff862f34fcaa6bbbed45d319f))
* add text layout integration tests ([e66bf30](https://github.com/aram-devdocs/GoudEngine/commit/e66bf30ebe64cb730b3e52143d451a12f23d1e24))
* add text rendering API, layout engine, and bitmap font support ([d55228f](https://github.com/aram-devdocs/GoudEngine/commit/d55228fbf8935f3ed269bfc2d6b0201ed662fa18))
* add timestep configuration FFI exports ([1a261f7](https://github.com/aram-devdocs/GoudEngine/commit/1a261f7fff3948c97598cb4428dfbc89d77fd2d6)), closes [#275](https://github.com/aram-devdocs/GoudEngine/issues/275)
* add transform delta coverage in serialization ([8fb73e9](https://github.com/aram-devdocs/GoudEngine/commit/8fb73e9510030cda0a34fd15e468b22dcd939bde))
* add UI component system with separate node tree ([64abd5f](https://github.com/aram-devdocs/GoudEngine/commit/64abd5fd831a21f2146a72274d254c475a7619b9))
* add UI component system with separate node tree ([#233](https://github.com/aram-devdocs/GoudEngine/issues/233)) ([16f520d](https://github.com/aram-devdocs/GoudEngine/commit/16f520d8c0a6d9d59e67471bb2722cf5882f8f0f))
* add UI layout engine and input semantics ([#480](https://github.com/aram-devdocs/GoudEngine/issues/480)) ([aa7964e](https://github.com/aram-devdocs/GoudEngine/commit/aa7964e397950adb2fcb71143ba515bbfbcabeaf))
* add UI widgets and theming ([253e743](https://github.com/aram-devdocs/GoudEngine/commit/253e74304dcf0587a0c2b82ece598e077b9fca19))
* add UI widgets and theming ([389e86b](https://github.com/aram-devdocs/GoudEngine/commit/389e86bb324ab3e559f6847499f2b825c4c89252))
* add virtual filesystem abstraction for asset loading ([7732fef](https://github.com/aram-devdocs/GoudEngine/commit/7732fef98ad8286815cd156d20300e0abc6604ff))
* align python network packet contract test ([597a89b](https://github.com/aram-devdocs/GoudEngine/commit/597a89b74c1286c7343376ba676bc816d75c11f9))
* complete audio sdk parity ([e79f81c](https://github.com/aram-devdocs/GoudEngine/commit/e79f81cbea0040bc7f4d1c3108150ee1c73c3093))
* complete audio SDK parity ([af36f94](https://github.com/aram-devdocs/GoudEngine/commit/af36f94642ac015fc675a3649d681956872cffc6))
* complete audio system — manager integration, per-channel volume, streaming ([1288573](https://github.com/aram-devdocs/GoudEngine/commit/128857302f6ff991aa347498fcfef7347921f2cd))
* complete audio system — manager integration, per-channel volume, streaming ([2283dab](https://github.com/aram-devdocs/GoudEngine/commit/2283dabe9ab5690a0eb9bbe9a891448f2635b036))
* complete physics step, gravity, and collision response systems ([605c53d](https://github.com/aram-devdocs/GoudEngine/commit/605c53de86e6b39dfeb093493cf08f21b4a967d4))
* **csharp:** add handwritten networking wrapper API ([ff37ac1](https://github.com/aram-devdocs/GoudEngine/commit/ff37ac1573031ce8fcad9d24f8c1e1c660233e83))
* debugger-first provider diagnostics + timeline recording (Phase 2.5.0) ([1ba8f81](https://github.com/aram-devdocs/GoudEngine/commit/1ba8f810f980541cacf1350b9364ae71f72af842))
* debugger-first provider diagnostics architecture (Phase 2.5.0) ([68ee183](https://github.com/aram-devdocs/GoudEngine/commit/68ee1835eb6e84ec427564b8ae4fe59198e1287e))
* enable debugger in all sandbox apps and add MCP E2E test script ([01f70d1](https://github.com/aram-devdocs/GoudEngine/commit/01f70d1c19929ccfb641a16280a150c70cca7d3c))
* expose animation control APIs via FFI and SDKs ([#477](https://github.com/aram-devdocs/GoudEngine/issues/477)) ([5279ba6](https://github.com/aram-devdocs/GoudEngine/commit/5279ba6390d94549cca3be11e243e4f9c5c8cee2))
* expose debugger runtime through ffi and sdks ([4113b51](https://github.com/aram-devdocs/GoudEngine/commit/4113b518311b48ad1aab1c82d1e13fe26a6192c1))
* expose network ffi controls and stats ([c4e3bcd](https://github.com/aram-devdocs/GoudEngine/commit/c4e3bcd3c0c88343f7f9d188cfd0ed0013af50f2))
* expose scene load/unload via FFI and SDK wrappers ([#476](https://github.com/aram-devdocs/GoudEngine/issues/476)) ([ccbecd7](https://github.com/aram-devdocs/GoudEngine/commit/ccbecd7536df0839fa3e8a8bc898844f9392621e))
* expose text rendering via FFI with SDK and unicode support ([#479](https://github.com/aram-devdocs/GoudEngine/issues/479)) ([ca74335](https://github.com/aram-devdocs/GoudEngine/commit/ca743353d4c276d8d2608d30e44307169174cde3))
* generate sdk networking accessors and stats ([127eec9](https://github.com/aram-devdocs/GoudEngine/commit/127eec92ca8966fb139da4046b7b1965a23a6289))
* implement collision events, sensors, raycast masks, and layer filtering ([#478](https://github.com/aram-devdocs/GoudEngine/issues/478)) ([d4e34f9](https://github.com/aram-devdocs/GoudEngine/commit/d4e34f9d241680ed0ed328e5bcceb1bf805b555d))
* persist debugger runtime artifacts ([00244b6](https://github.com/aram-devdocs/GoudEngine/commit/00244b653177f673818bb66bc6fba4934821bd05))
* preserve networking peer ids in sdk codegen ([da6ce48](https://github.com/aram-devdocs/GoudEngine/commit/da6ce48e2008776d031bc86c6304344cac47e70a))
* **python-sdk:** add handwritten networking wrapper layer ([969bb83](https://github.com/aram-devdocs/GoudEngine/commit/969bb831215e4bd9fe7c2f367bd7a08eeb6e4061))
* roll out debugger public surface prompts and Rust guidance ([31d3f99](https://github.com/aram-devdocs/GoudEngine/commit/31d3f99ead9a2d48e7050b4d503053f6ee484871))
* roll out feature lab debugger attach workflow ([fbebeef](https://github.com/aram-devdocs/GoudEngine/commit/fbebeef3a0b7f70d089e9234e1c7c31395ba3c78))
* spatial audio, WASM WebAudio provider, and mixing controls ([#481](https://github.com/aram-devdocs/GoudEngine/issues/481)) ([afe4ff6](https://github.com/aram-devdocs/GoudEngine/commit/afe4ff650c77f172d8e2bf5133035941417cec9f))
* **typescript:** add handwritten networking wrapper layer ([d4fcd90](https://github.com/aram-devdocs/GoudEngine/commit/d4fcd90e629865c084394259dca9f2e2bb886d7a))
* wire native network debug overlay runtime seam ([6b0f8d8](https://github.com/aram-devdocs/GoudEngine/commit/6b0f8d876398c0bf5ad0dc88569fdf544962a3fd))
* wire renderer3d debug draw through runtime ([f694df3](https://github.com/aram-devdocs/GoudEngine/commit/f694df3cf872ef05ca128afba50e5a1d3d82020c))


### Bug Fixes

* add assertions to log_error test per code quality review ([7de208c](https://github.com/aram-devdocs/GoudEngine/commit/7de208c58ad440f1c76590a847704f0b57f9e3b3))
* add audio FFI to build manifest and codegen schema ([be670a3](https://github.com/aram-devdocs/GoudEngine/commit/be670a30960bab4f9cf2902a827efff7b79b0e58))
* add AudioManager to window lifecycle and cleanup_finished FFI ([2c25cd0](https://github.com/aram-devdocs/GoudEngine/commit/2c25cd059992cc8f6aa61fca7d685933c664b08c))
* add bytes→Buffer type mapping and audio stubs for WASM TS build ([2e047f2](https://github.com/aram-devdocs/GoudEngine/commit/2e047f25896cb09e188c122f51687ead23dbc40c))
* add checkHotSwapShortcut to codegen schema and fix C#/WASM generation ([ca50319](https://github.com/aram-devdocs/GoudEngine/commit/ca50319d500153a5f285a7afb8688e77d7a8d5ea))
* add FfiText pointer mappings to C# codegen ([ca94d36](https://github.com/aram-devdocs/GoudEngine/commit/ca94d362ee6fda72fc934bea95a6b193c0203063))
* add missing integration and lifecycle tests for asset pipeline features ([388a2f1](https://github.com/aram-devdocs/GoudEngine/commit/388a2f160353bfa5e8a7eccc4cf76bd10789f073))
* add native audio activate ffi export ([329e2a5](https://github.com/aram-devdocs/GoudEngine/commit/329e2a59524e593c43d2bf445b464f0c3bd97fb4))
* add negative duration test and fix DllImport type for TransitionType ([b8a443d](https://github.com/aram-devdocs/GoudEngine/commit/b8a443d9a2b51d65d4a314395acbd1855cae3fa2))
* add provider capability stubs to WASM TypeScript codegen ([77ca31b](https://github.com/aram-devdocs/GoudEngine/commit/77ca31bd7365ffb036cab417a6876e8661dccdba))
* add providers.rs to FFI manifest source list ([dc52d85](https://github.com/aram-devdocs/GoudEngine/commit/dc52d85ba75d88a73ed5de305c80228704789804))
* add sandbox typescript lockfile ([c567d95](https://github.com/aram-devdocs/GoudEngine/commit/c567d950a8ce95b7aac65f55bb88a2cd467b3e38))
* add timestep FFI exports and behavioral physics tests ([fdad6dd](https://github.com/aram-devdocs/GoudEngine/commit/fdad6dd3750cf5e3301c00cd817d3add67640b2e))
* add TransitionType enum to codegen schema for SDK type safety ([719372f](https://github.com/aram-devdocs/GoudEngine/commit/719372f1c1ce5fa0aadaa20aeeed24eb6a88c52f))
* add TS web SDK animation stubs and fix napi entity type ([1f424de](https://github.com/aram-devdocs/GoudEngine/commit/1f424deeb25fcbc7fe0ddafcb74b7a8de3626b19))
* add ui manager to ts ci shim ([e39649e](https://github.com/aram-devdocs/GoudEngine/commit/e39649e42ee21300e61b746feb6d4cbe7d7b6fdc))
* address AI code review warnings — hot-reload docs, texture fallback warning, glyph_provider tests, cache duplication comment ([b0fb83b](https://github.com/aram-devdocs/GoudEngine/commit/b0fb83b5eded814c3722b171cb1581ca0dc124c1))
* address AI code review warnings (W1-W4) ([1bc8dd1](https://github.com/aram-devdocs/GoudEngine/commit/1bc8dd18033cf3a14bf28136ada3d917babc6137))
* address AI code review warnings for serialization framework ([cf2266a](https://github.com/aram-devdocs/GoudEngine/commit/cf2266aee7ea7b806bb4920a9174d237ada572d0))
* address all review warnings and remaining silent FFI error paths ([a672de4](https://github.com/aram-devdocs/GoudEngine/commit/a672de4081f56a987789799adf8d0f115bda2f56))
* address Claude code review — TS audio bindings, mutex safety, metadata ([488f4a8](https://github.com/aram-devdocs/GoudEngine/commit/488f4a89fbd1cb6ebd37c0d2e6192c9bf1cfda65))
* address claude review warnings ([496d37f](https://github.com/aram-devdocs/GoudEngine/commit/496d37f73a117946118799b95a8d78e4edb49191))
* address code review — remove nested unsafe, validate timestep FFI, fix stale doc ([3a476a3](https://github.com/aram-devdocs/GoudEngine/commit/3a476a31862b4466d88960e44c9af0e9b53710cb))
* address PR 509 networking review follow-up ([2b49a1c](https://github.com/aram-devdocs/GoudEngine/commit/2b49a1ccedee5aa18862c7d43bd9186d8ded4cc4))
* address PR review — FFI API fixes, TS SDK parity, dead code cleanup ([414fb65](https://github.com/aram-devdocs/GoudEngine/commit/414fb65d24df3bbad59e80c7334e183a46e22367))
* address PR review — guard destroy_scene, expose TransitionComplete, track TransitionType.g.cs ([11c3429](https://github.com/aram-devdocs/GoudEngine/commit/11c34292f4baa14494e51a1b02c87fe1a7a3995c))
* address review warnings and add mesh/audio fallbacks ([c0772c2](https://github.com/aram-devdocs/GoudEngine/commit/c0772c2539441e4e84dff3d3ec6ecd2668185d93))
* address spec review — wire diagnostic config, fix remaining silent FFI paths ([eba5e5d](https://github.com/aram-devdocs/GoudEngine/commit/eba5e5d53c575f70d889851f532e30b7b68c09bf))
* address ui manager review warnings ([d83b9f1](https://github.com/aram-devdocs/GoudEngine/commit/d83b9f1caf2785b72515a2f75b5aa68b8367334f))
* address UI review feedback ([aa4070b](https://github.com/aram-devdocs/GoudEngine/commit/aa4070b9dbd37b2bdcabe4f82318ad84cb8d1edf))
* align sandbox contract across sdk examples ([95836ca](https://github.com/aram-devdocs/GoudEngine/commit/95836caba60713bccfd37a63007d9292778095a4))
* align sandbox layout across runtimes ([025407f](https://github.com/aram-devdocs/GoudEngine/commit/025407fa1790b797855def1dad54c306ad5a61aa))
* allow nondeterministic docs artifacts in clean-room checks ([7cc41b3](https://github.com/aram-devdocs/GoudEngine/commit/7cc41b363b7109a229919e543f1d6f8d16a9ba0f))
* apply networking review fixes ([3c798e6](https://github.com/aram-devdocs/GoudEngine/commit/3c798e666c74fa60b063f5e04227f4e990f8e97b))
* avoid debugger artifact path panic ([9247ec4](https://github.com/aram-devdocs/GoudEngine/commit/9247ec443d6208ae4ec3ad6c968171832c2d2649))
* batch 2.5 hardening follow-up ([#498](https://github.com/aram-devdocs/GoudEngine/issues/498)) ([22fbf92](https://github.com/aram-devdocs/GoudEngine/commit/22fbf92cbd77519714d847c155ae7f1a9214cf21))
* bootstrap parity artifacts before sdk smoke tests ([1a59811](https://github.com/aram-devdocs/GoudEngine/commit/1a598113154ad38c99c54c3e8fcd67916705b152))
* bootstrap typescript clean-room docs build ([cd74ca3](https://github.com/aram-devdocs/GoudEngine/commit/cd74ca3777b907b215c0be62298d711d40844e3c))
* bootstrap typescript examples from source ([a5c8282](https://github.com/aram-devdocs/GoudEngine/commit/a5c8282f7e792e1a496d4deb20874de4bd0d17e9))
* build local typescript sdk before examples ([a94c30f](https://github.com/aram-devdocs/GoudEngine/commit/a94c30fe94aa32a775d0d678bdd63abea18683d0))
* **ci:** marshal csharp bytes params as ptr+len in codegen ([c1dd793](https://github.com/aram-devdocs/GoudEngine/commit/c1dd793d69249e9c49ba7fd5648a6246d253c618))
* **ci:** regenerate sdk files to resolve codegen drift ([373fb52](https://github.com/aram-devdocs/GoudEngine/commit/373fb52b8f0787253c9a716694f0dfe0ffaa6f8e))
* clarify custom transition design and fix misleading test comment ([eda2793](https://github.com/aram-devdocs/GoudEngine/commit/eda2793cbb47ce6ca893810e7c40d669cf4303cb))
* clippy and fmt issues ([408f25a](https://github.com/aram-devdocs/GoudEngine/commit/408f25a84d4815198f7bfc04b60fdc95eca21e4b))
* commit regenerated python sdk artifacts ([4945d2c](https://github.com/aram-devdocs/GoudEngine/commit/4945d2c7cdf125948db1bb82f2dc07ce57289843))
* complete alpha-001 phase 0-2 remediation ([d0ac9b9](https://github.com/aram-devdocs/GoudEngine/commit/d0ac9b9eaa8425a138b9cb58a23d7e67c803436b))
* complete alpha-001 phase 0-2 remediation ([45aa9ae](https://github.com/aram-devdocs/GoudEngine/commit/45aa9aea97638896a71bd626b3e6fe07a224ec51))
* complete ALPHA-001 phases 0-2.5 audit remediation ([b04a0c9](https://github.com/aram-devdocs/GoudEngine/commit/b04a0c93d4b0eef46dac8b894782c57660cb3278))
* complete ALPHA-001 phases 0-2.5 audit remediation ([5666197](https://github.com/aram-devdocs/GoudEngine/commit/56661979230ad97e4e2b0707d522db58a8b7240c))
* consolidate env var diagnostic tests to eliminate race condition ([0f1068c](https://github.com/aram-devdocs/GoudEngine/commit/0f1068c88b4f00ae926b1cac7231600bbcc420f5))
* correct benchmark match arms for all types in json_vs_binary_size ([9d9fbb1](https://github.com/aram-devdocs/GoudEngine/commit/9d9fbb1cc7ecda6fdb88e53ce04286aa3af69bb1))
* correct csharp ui manager test paths ([69089a4](https://github.com/aram-devdocs/GoudEngine/commit/69089a4b4ddd2459c8f9d3a7d94af7055f52498e))
* correct import ordering in logging_tests.rs for rustfmt ([ee263ed](https://github.com/aram-devdocs/GoudEngine/commit/ee263ed54ae1cd98084dbd9f38fb65156d2c2f0b))
* correct packager doctest to use Path::new() instead of &str ([8caa3fc](https://github.com/aram-devdocs/GoudEngine/commit/8caa3fc5e45da1989b49a7c34e77e27963aa4215))
* correct SAFETY comment on AudioManager Send/Sync impls ([d6270ef](https://github.com/aram-devdocs/GoudEngine/commit/d6270ef5dfb629be1d4f98938d61b650e14d8ac2))
* debugger IPC pipeline for MCP clients — snapshot, capture, socket path, frame limit ([e90cbaf](https://github.com/aram-devdocs/GoudEngine/commit/e90cbaf36376722f44b2adba319fbfee9851cffa))
* **deps:** consolidate dependabot updates with compatibility fixes ([#497](https://github.com/aram-devdocs/GoudEngine/issues/497)) ([4ca1597](https://github.com/aram-devdocs/GoudEngine/commit/4ca15970881d2f0e6b7291161669e9e1dabef4d2))
* eliminate race condition in MCP test harness ([2c0f7c3](https://github.com/aram-devdocs/GoudEngine/commit/2c0f7c33588cfa9a12680aaf5219f290cd7a9edc))
* emit python ffi enum fields as scalars ([97103fa](https://github.com/aram-devdocs/GoudEngine/commit/97103fa70dd55d26500d3b1eee11d65c3ba11d16))
* exempt generated *.g.rs from line limit, split TS audio into audio.g.rs ([f69a4bd](https://github.com/aram-devdocs/GoudEngine/commit/f69a4bdcbf1bc31a385410aa619794e401db6b42))
* export TransitionType from Python SDK generated __init__.py ([43c2ae4](https://github.com/aram-devdocs/GoudEngine/commit/43c2ae44e091ce84a6af13a24d8259b069d4b715))
* format ui ffi and wasm updates ([14ebc2f](https://github.com/aram-devdocs/GoudEngine/commit/14ebc2f69c35c0b751cf04ed2eb181dfe423d0e9))
* format UI review follow-up ([c63a204](https://github.com/aram-devdocs/GoudEngine/commit/c63a204374f5763cd7c5dde6463f0f8e85a1f9bd))
* format wasm ui bindings ([0101872](https://github.com/aram-devdocs/GoudEngine/commit/01018729902abeab6e2877a3f3b97bf4e5121ed0))
* gate platform-specific code to fix Windows clippy on main ([759286b](https://github.com/aram-devdocs/GoudEngine/commit/759286bfcb8b4c28263e13e5871f9e0238219328))
* gate Windows-unused imports and functions with cfg(not(windows)) ([eca02b4](https://github.com/aram-devdocs/GoudEngine/commit/eca02b4968c9decbf5a5f75267921654f601cc24))
* generate TypeScript interfaces for provider capability types ([552120c](https://github.com/aram-devdocs/GoudEngine/commit/552120ca8aacc81a37e8397aecd9cee5bded919b))
* guard typescript feature lab transform fallback ([44cf755](https://github.com/aram-devdocs/GoudEngine/commit/44cf755288802c4df7a521689046e4268178fbea))
* handle BC compressed texture formats in wgpu backend match ([ac21577](https://github.com/aram-devdocs/GoudEngine/commit/ac21577f8febbb30557be9730a3995eca2759f58))
* harden regen validation and ws provider layout ([0e567e1](https://github.com/aram-devdocs/GoudEngine/commit/0e567e14db783717adcd05eebe189494c02adf6e))
* harden rust sandbox 3d mode ([ccfc385](https://github.com/aram-devdocs/GoudEngine/commit/ccfc385a743bc62a5e39f387d3c5f828e78665c4))
* install wasm-pack portably on macos parity lane ([7187ea3](https://github.com/aram-devdocs/GoudEngine/commit/7187ea34613fae10ecb901c1f527db5fbfa6a12a))
* keep rust feature lab attach route alive ([07205e5](https://github.com/aram-devdocs/GoudEngine/commit/07205e54a1a2ee04f3e56f0d483f1b1d421b6ec2))
* load bitmap font texture data and refactor draw_text helpers ([bd2d387](https://github.com/aram-devdocs/GoudEngine/commit/bd2d3877094ece8ac171185e75db7c61a8e433ec))
* make python sdk init reproducible ([84e114d](https://github.com/aram-devdocs/GoudEngine/commit/84e114d7aeed6710016a93d4dc1e4188dddb3514))
* move Transform2D serialization tests to ecs layer to satisfy lint-layers ([a377902](https://github.com/aram-devdocs/GoudEngine/commit/a377902c66326c68cdd3803789d9787971a657d6))
* narrow network ffi unsafe pointer blocks ([479b23a](https://github.com/aram-devdocs/GoudEngine/commit/479b23af254012aef70f0cb299582e81b7557b9d))
* **python-sdk:** align networking packet and host behavior ([837fef2](https://github.com/aram-devdocs/GoudEngine/commit/837fef20325821abd3f5b35efa6a86bcc1e97e46))
* **python-sdk:** round out networking endpoint helpers ([bf4e6d1](https://github.com/aram-devdocs/GoudEngine/commit/bf4e6d1ec44d9a570d2dd51f7ef005f1f8140908))
* recover sandbox native parity ([226e3fe](https://github.com/aram-devdocs/GoudEngine/commit/226e3fec8e11336c8a0940b4854e4dc73d6a955b))
* recover sandbox parity ([0139a99](https://github.com/aram-devdocs/GoudEngine/commit/0139a99803850e44dcd54672dd800513c706ca55))
* reduce animation controller.rs below 500-line limit ([9575f01](https://github.com/aram-devdocs/GoudEngine/commit/9575f018a9ad94994f94058c6ade17a7ff246cc3))
* reduce sandbox startup drift ([1fd25b1](https://github.com/aram-devdocs/GoudEngine/commit/1fd25b14794172e2eeb854e18e1904e4021e0e0b))
* reduce serialization benchmarks to under 500 lines ([bc75500](https://github.com/aram-devdocs/GoudEngine/commit/bc75500167d07bf5358ea18b77402932de26fad0))
* regenerate TypeScript napi game.g.rs to fix codegen drift ([89d694d](https://github.com/aram-devdocs/GoudEngine/commit/89d694dbd959adfd3b7b13b95578d7b742c949e7))
* reject negative network host handles in SDK wrappers ([6cdaaec](https://github.com/aram-devdocs/GoudEngine/commit/6cdaaeca5b44f552c234ddbd4a0b886934e31773))
* remove duplicate resolve_body/resolve_collider from rapier3d queries ([d60e529](https://github.com/aram-devdocs/GoudEngine/commit/d60e5294512afa4669d4d66c6ac8b9041aad34a6))
* remove overlay sdk state from engine networking batch ([2a8b476](https://github.com/aram-devdocs/GoudEngine/commit/2a8b476186e3abd4581e72dec4308a8715f0327a))
* remove unreachable block in python network contract test ([3dd4585](https://github.com/aram-devdocs/GoudEngine/commit/3dd4585129d03cfad6443003d531c57eb00ae74a))
* rename reserved keyword param and add debugger methods to WasmGameHandle ([9421307](https://github.com/aram-devdocs/GoudEngine/commit/942130772ba7adc69134ade97891b8de975f9f25))
* repair csharp codegen split imports ([7f41fcc](https://github.com/aram-devdocs/GoudEngine/commit/7f41fcc7d4266b5eae8a1685fc15fbaf5dbc95f4))
* repair gen_python.py syntax error from merge conflict resolution ([2c62aa2](https://github.com/aram-devdocs/GoudEngine/commit/2c62aa216bbcfe5fdfee2fe1d0fb4a7579ce4ce7))
* repair sandbox peer smoke readiness ([da43132](https://github.com/aram-devdocs/GoudEngine/commit/da431322959be536ba67bb574558a23b85718a94))
* resolve enum underlying types in Python FFI codegen to prevent ABI mismatch ([17d385d](https://github.com/aram-devdocs/GoudEngine/commit/17d385d306e07d9594c829bb05979cf9fe4f01c5))
* resolve ffi_manifest.json merge conflict with main ([6115544](https://github.com/aram-devdocs/GoudEngine/commit/6115544811ef21d7a98ece194fb2c2518f925542))
* resolve ffi_manifest.json merge conflict with main ([f9011f4](https://github.com/aram-devdocs/GoudEngine/commit/f9011f442bd9ba64e51df8757f04ad0bb726f757))
* resolve layer violations by moving TextAlignment to core types ([1b3bb31](https://github.com/aram-devdocs/GoudEngine/commit/1b3bb3141d09886b994516b88e824ecc61c35631))
* resolve merge conflict in ffi_manifest.json after batch 2.4 merges ([3b6a83f](https://github.com/aram-devdocs/GoudEngine/commit/3b6a83ffc06bcff51d3e3e4df8c68b27c7ed5767))
* resolve merge conflicts with main (codegen + TS generated files) ([d5ca2d8](https://github.com/aram-devdocs/GoudEngine/commit/d5ca2d83e2dc0a03521b8acdeba0cea8f8cc1ad9))
* resolve merge conflicts with main (instance.rs, codegen, SDK generated) ([8713503](https://github.com/aram-devdocs/GoudEngine/commit/87135031c6b60ef3f539b774b052e9da509f68a7))
* resolve merge conflicts with main (schema, codegen, generated keys) ([3804fd8](https://github.com/aram-devdocs/GoudEngine/commit/3804fd814e6009ec861f457d44afbf8ebde730d2))
* resolve merge conflicts with main for batch 2.4 final merge ([b9d5734](https://github.com/aram-devdocs/GoudEngine/commit/b9d5734a00d3ff760c16d9faa0aeda1531dd4bbf))
* resolve rustfmt formatting issues in registry.rs and instance.rs ([7c28a3b](https://github.com/aram-devdocs/GoudEngine/commit/7c28a3beafa1686a428698df68b68a649334a328))
* restore community stats updates on protected main ([c2d3578](https://github.com/aram-devdocs/GoudEngine/commit/c2d3578ceeca8dcf5e290dd3d0e5fa17aa464322))
* restore community stats workflow on protected main ([b962f48](https://github.com/aram-devdocs/GoudEngine/commit/b962f4856e46238ba29048fd5ff332bc5619aa66))
* restore generated sdk drift and wasm build ([6a16682](https://github.com/aram-devdocs/GoudEngine/commit/6a16682f2bf22a5ff0117660fd05b5212cd4d8dd))
* restore gh-issue script executability ([54a78ff](https://github.com/aram-devdocs/GoudEngine/commit/54a78ff6a9f5853c79e148bf027ec2212ceef16f))
* restore nested agents guidance ([c1b8c05](https://github.com/aram-devdocs/GoudEngine/commit/c1b8c05af06370c67ec45b210083483dff7ec089))
* sandbox recovery checkpoint before second audit batch ([a80f126](https://github.com/aram-devdocs/GoudEngine/commit/a80f1267e5dc9735bc0260ceb937bc2176306529))
* satisfy ai config validation ([1c9aa78](https://github.com/aram-devdocs/GoudEngine/commit/1c9aa789ed8c2cac052996d77026e8b294f0c826))
* satisfy clippy review gates on ci ([02162c1](https://github.com/aram-devdocs/GoudEngine/commit/02162c168b0a804634f47accc85fec52c6437e42))
* satisfy layer lint for state sync ([2ed8967](https://github.com/aram-devdocs/GoudEngine/commit/2ed89675c31c97317c866e42820608f274241a5a))
* serialize diagnostic tests with mutex to prevent parallel test races ([7a527b4](https://github.com/aram-devdocs/GoudEngine/commit/7a527b41705c460c72cf7b406d8902a4ea945c45))
* simplify shared agent orchestration ([7dd2bb2](https://github.com/aram-devdocs/GoudEngine/commit/7dd2bb2309981ddbb276d5ef27d4cb9d70ac1b55))
* split GoudGame provider methods into separate module ([eb24c96](https://github.com/aram-devdocs/GoudEngine/commit/eb24c96beca83e7b47aaafa753d522367f902a37))
* split oversized test and provider files to stay under 500 lines ([ba1c5ba](https://github.com/aram-devdocs/GoudEngine/commit/ba1c5bacfe9c8a1134274e72a7e63356c572eb95))
* split physics modules for ci ([f1070bd](https://github.com/aram-devdocs/GoudEngine/commit/f1070bd967317716ecbd13c51f8c9baca533e85a))
* split ref counting into separate module and remove expect() from archive format ([10cc099](https://github.com/aram-devdocs/GoudEngine/commit/10cc0990c95fa69186be358c78cbaa4d217dc8ec))
* stabilize ci artifact and runtime checks ([d1711f1](https://github.com/aram-devdocs/GoudEngine/commit/d1711f1dabd77f22fed52f5ad3b82292235ed9da))
* stabilize codex agent concurrency config ([9bb4ac5](https://github.com/aram-devdocs/GoudEngine/commit/9bb4ac56db40a129744a3679b486503245f0a2f7))
* stabilize native network overlay handle sync ([30913bb](https://github.com/aram-devdocs/GoudEngine/commit/30913bbc3c60d4d30fc613b63fdde0291b867dad))
* stabilize sandbox smoke and rust text validation ([133ed15](https://github.com/aram-devdocs/GoudEngine/commit/133ed15321ca1d29739cbe958f022d4a29c50aba))
* stabilize typescript wasm release pipeline ([c5979fb](https://github.com/aram-devdocs/GoudEngine/commit/c5979fb59671b9611376a60df98196bb62753d72))
* sync web typescript generation ([699aab1](https://github.com/aram-devdocs/GoudEngine/commit/699aab1042275730c82434092045354544aa9285))
* text baseline alignment and missing sandbox body text ([c9bd5b5](https://github.com/aram-devdocs/GoudEngine/commit/c9bd5b5025a030c649e069e07e8a40a94f8e7236))
* tighten debugger ffi and ts typings ([33859b7](https://github.com/aram-devdocs/GoudEngine/commit/33859b784f3bc57e4264fae55148cb7435c761c6))
* track debugger generated artifacts ([fae0411](https://github.com/aram-devdocs/GoudEngine/commit/fae04119e97adbbfd34a366fa95039fa8cd0d8b7))
* track feature lab example lockfile ([20c2bac](https://github.com/aram-devdocs/GoudEngine/commit/20c2bac29509e0640956ae738557aff2eba5d78d))
* track parallel execution module ([3eda6f8](https://github.com/aram-devdocs/GoudEngine/commit/3eda6f88c5745d4709ae0df112f5c58dbb8db75d))
* trim state.rs under 500 lines, fix instance.rs exempt pattern ([10741a9](https://github.com/aram-devdocs/GoudEngine/commit/10741a9425b46648c09876fb79299651dbb569ff))
* unblock final sdk parity ci lanes ([6269ed3](https://github.com/aram-devdocs/GoudEngine/commit/6269ed3208a70bc8a9a4176795849918db0da23f))
* update codegen for provider capability types ([db1fcc9](https://github.com/aram-devdocs/GoudEngine/commit/db1fcc93cf6765c58269ce0ab524d83bc0dd9e32))
* use AudioData::InMemory in fallback registry to match AudioAsset API ([95a15a7](https://github.com/aram-devdocs/GoudEngine/commit/95a15a73bcad5c571833b00da52e001ffadf177c))
* WASM build and codegen drift CI failures ([b6c2f44](https://github.com/aram-devdocs/GoudEngine/commit/b6c2f446082c6c8dcde7e8aae2845235cb62f070))
* wire bitmap fonts into text pipeline, add kerning support, and integration tests ([504109d](https://github.com/aram-devdocs/GoudEngine/commit/504109d919dcd6ca745a5943c41405b4798422ab))
* wire csharp smoke to local native lib ([735cced](https://github.com/aram-devdocs/GoudEngine/commit/735cced1b9f094a8d6ab63104ec977993f537a80))
* wire init_diagnostic_from_env into game init, improve log tests ([c8a3527](https://github.com/aram-devdocs/GoudEngine/commit/c8a35274800379dde88cf10806f43bed4574ddb6))
* wrap native network providers with debug sim and add ffi tests ([259c140](https://github.com/aram-devdocs/GoudEngine/commit/259c140fc00577a7a833d4d31f5097a16240348e))


### Refactoring

* extract transition methods from instance.rs to stay under 500-line limit ([8de4832](https://github.com/aram-devdocs/GoudEngine/commit/8de48328623f29d5a4612b6e7bcb44b50a1914ae))
* **ffi:** split networking ffi module ([e913dbc](https://github.com/aram-devdocs/GoudEngine/commit/e913dbccceecd4585c034a1efd7c656906af67cb))
* migrate scene binary helpers to core serialization module ([bf511c3](https://github.com/aram-devdocs/GoudEngine/commit/bf511c396afbfb737dba97f79a2c2594f76d74f7))
* move audio activate tests out of ffi controls ([58a7aa6](https://github.com/aram-devdocs/GoudEngine/commit/58a7aa6899e55e4bebdecdf44e67d742389b1a3d))
* remove all line-limit exemptions by splitting instance/mod.rs ([420b211](https://github.com/aram-devdocs/GoudEngine/commit/420b2119c455afc132d76db04e556eb0509a83ae))
* restore strict gh-issue orchestration ([ed99f19](https://github.com/aram-devdocs/GoudEngine/commit/ed99f19d74032f7f3994c57d5edf3fa55b51586a))
* restore strict gh-issue orchestration ([0e494a5](https://github.com/aram-devdocs/GoudEngine/commit/0e494a597ef38a14bc86df48542b3e4c42dab40e))
* simplify agent control plane ([b594f26](https://github.com/aram-devdocs/GoudEngine/commit/b594f26a71b831e4c917e91e0a38e59ff65c3968))
* simplify agent control plane ([6eb72c2](https://github.com/aram-devdocs/GoudEngine/commit/6eb72c297f11366b677b29c5b4a64ad3ffb1d456))
* split audio ffi query helpers ([1dbff93](https://github.com/aram-devdocs/GoudEngine/commit/1dbff9373ae8e2af9627b40fcdef581a77ac3880))
* split debugger bridge helpers ([31f3fb8](https://github.com/aram-devdocs/GoudEngine/commit/31f3fb82b5b88e73b66ff345d039b0e13aecc275))
* split network transport helpers ([3fbbe60](https://github.com/aram-devdocs/GoudEngine/commit/3fbbe600337d317bb48e12ccd2c03d4792d77a47))
* split oversized Rust files to satisfy 500-line CI limit ([243a285](https://github.com/aram-devdocs/GoudEngine/commit/243a285bcec595377a814bdbce4e3ad52f30dcbe))
* split oversized sdk generator and test modules ([4bf4bd0](https://github.com/aram-devdocs/GoudEngine/commit/4bf4bd03b99a8535cf4567ffa4637cac9132887c))
* split text_batch tests into separate file to stay under 500-line limit ([d9bfa25](https://github.com/aram-devdocs/GoudEngine/commit/d9bfa25a14093d1afaa75c42bbf2716777646aec))
* split ts node generator ([10a840c](https://github.com/aram-devdocs/GoudEngine/commit/10a840c749c4000de2e42880b4ed4c4b78dae401))
* split wasm debugger attach stub ([b3ee72b](https://github.com/aram-devdocs/GoudEngine/commit/b3ee72bee697536fad9768be1150e0e657f987d7))
* unify Claude/Codex agent configs from canonical catalog ([#500](https://github.com/aram-devdocs/GoudEngine/issues/500)) ([56fd6be](https://github.com/aram-devdocs/GoudEngine/commit/56fd6be76ea11ca9ecc824edeee51f25eee3406f))

## [0.0.828](https://github.com/aram-devdocs/GoudEngine/compare/v0.0.827...v0.0.828) (2026-03-08)


### Features

* 3D transform propagation, entity cloning, plugin system ([#440](https://github.com/aram-devdocs/GoudEngine/issues/440)) ([8a41896](https://github.com/aram-devdocs/GoudEngine/commit/8a41896567788e91845c6cd2896e157c9e380a50))
* add animation controller, tween/easing library, and 2D skeletal animation ([#456](https://github.com/aram-devdocs/GoudEngine/issues/456)) ([b200ef1](https://github.com/aram-devdocs/GoudEngine/commit/b200ef17c27f41a2b3cb82aeb97e9a057d15943d))
* add async asset loading with background thread pool ([ae959d3](https://github.com/aram-devdocs/GoudEngine/commit/ae959d33e5f3da6ded9da968a00de5c2d8b56ce7)), closes [#195](https://github.com/aram-devdocs/GoudEngine/issues/195)
* add content pipeline asset types and dependency tracking ([#457](https://github.com/aram-devdocs/GoudEngine/issues/457)) ([ef4db6e](https://github.com/aram-devdocs/GoudEngine/commit/ef4db6ef6dd62b39f0b685b6c231f9cae2d4e81e))
* add default systems, non-send thread safety, and system set ordering ([#454](https://github.com/aram-devdocs/GoudEngine/issues/454)) ([f38e643](https://github.com/aram-devdocs/GoudEngine/commit/f38e64306d65dff791c267482db47e06c38e0eda))
* add EngineConfig builder with provider selection (F02-07) ([#453](https://github.com/aram-devdocs/GoudEngine/issues/453)) ([517de29](https://github.com/aram-devdocs/GoudEngine/commit/517de29e9d6aad8d43bb56089887f67ce8726f69))
* add error context propagation, recovery classification, and SDK error types ([#455](https://github.com/aram-devdocs/GoudEngine/issues/455)) ([b33ad1c](https://github.com/aram-devdocs/GoudEngine/commit/b33ad1cd392ee4d5b510f81dd920f20d21ecbf3a))
* add font asset loader for TTF and OTF files ([58f6e51](https://github.com/aram-devdocs/GoudEngine/commit/58f6e519235aca498d0f9c21fa82a2ae92ec1965)), closes [#150](https://github.com/aram-devdocs/GoudEngine/issues/150)
* add FPS stats debug overlay and sprite batch benchmarks ([#444](https://github.com/aram-devdocs/GoudEngine/issues/444)) ([13c7f67](https://github.com/aram-devdocs/GoudEngine/commit/13c7f67fb86115b1182d3a61e0ec5f82c9824c0d))
* add glyph atlas generation and caching ([#451](https://github.com/aram-devdocs/GoudEngine/issues/451)) ([9714a28](https://github.com/aram-devdocs/GoudEngine/commit/9714a28491351d47d468c49f52f8f4663c121e80))
* add Rapier2D and Rapier3D physics providers ([#458](https://github.com/aram-devdocs/GoudEngine/issues/458)) ([9361d4a](https://github.com/aram-devdocs/GoudEngine/commit/9361d4af0486c47b3ba92f4845db121dceac2fc9))
* add scene serialization, loading/unloading, and prefab system ([#459](https://github.com/aram-devdocs/GoudEngine/issues/459)) ([351652f](https://github.com/aram-devdocs/GoudEngine/commit/351652f0a1df63fcf49f4f2005772391d0ab2c34))
* add SceneManager for multiple worlds/scenes support ([#442](https://github.com/aram-devdocs/GoudEngine/issues/442)) ([a4ad5ef](https://github.com/aram-devdocs/GoudEngine/commit/a4ad5ef54b0d5d5fb6d8dc31d8e369db1a7bb235))
* add sprite sheet animation component and system ([#445](https://github.com/aram-devdocs/GoudEngine/issues/445)) ([b36448c](https://github.com/aram-devdocs/GoudEngine/commit/b36448c0aad1dc224a6209ef38ef50f6e40c4dbd))
* add UDP and WebSocket transport layers ([#460](https://github.com/aram-devdocs/GoudEngine/issues/460)) ([2c0ec33](https://github.com/aram-devdocs/GoudEngine/commit/2c0ec33cb740954abf7dda71ae414e77a157905f))
* **ecs:** implement change detection (Changed&lt;T&gt;, Added&lt;T&gt;) ([b04334c](https://github.com/aram-devdocs/GoudEngine/commit/b04334c9b5d478d8c8dac97cbaeb19d5b8d5b047)), closes [#164](https://github.com/aram-devdocs/GoudEngine/issues/164)
* **ecs:** implement event system parameters (EventReader/EventWriter) ([378d595](https://github.com/aram-devdocs/GoudEngine/commit/378d595aa4247c0bf400f2dc4385995f8c672d0a)), closes [#165](https://github.com/aram-devdocs/GoudEngine/issues/165)
* **ecs:** implement optional component queries (Option&lt;&T&gt;) ([e4e6b55](https://github.com/aram-devdocs/GoudEngine/commit/e4e6b558745165ac3b68983c02910f99adeaaa79))
* **ecs:** implement query archetype caching ([d879835](https://github.com/aram-devdocs/GoudEngine/commit/d879835d0b6df4f81da61392f542ead7bcd5430c))
* **error:** add from_error_code reverse mapping and recovery guidance docs ([507f090](https://github.com/aram-devdocs/GoudEngine/commit/507f09027195077a2267c7ef59a11422bb9e5529))
* headless renderer, FFI safety tests, and integration test suite ([#452](https://github.com/aram-devdocs/GoudEngine/issues/452)) ([a92fd5d](https://github.com/aram-devdocs/GoudEngine/commit/a92fd5d334dda93a5a035b87860ab9c21949fa04))
* implement audio asset loader with rodio decoding ([#450](https://github.com/aram-devdocs/GoudEngine/issues/450)) ([a4f3d10](https://github.com/aram-devdocs/GoudEngine/commit/a4f3d1099ef0823b56c09c6caf0e52fa1e7b747a))
* implement provider traits for all engine subsystems (F02-02 through F02-06) ([#447](https://github.com/aram-devdocs/GoudEngine/issues/447)) ([021367e](https://github.com/aram-devdocs/GoudEngine/commit/021367e6ecc13d73a280ee6a66277c5d14b76707))
* **sdk:** add error query function bindings to Python and C# SDKs ([a5ac1a1](https://github.com/aram-devdocs/GoudEngine/commit/a5ac1a1e97425552b957016209e23ddbe0b06b8b))


### Bug Fixes

* add workflow_dispatch trigger to RFC auto-approve ([2c407e9](https://github.com/aram-devdocs/GoudEngine/commit/2c407e9a2b3e4478fdc5c8b6ad75182ad518e798))
* address Claude code review warnings ([83b0a4d](https://github.com/aram-devdocs/GoudEngine/commit/83b0a4de58d779628e16e3f50a2e21af96e71b6f))
* address Claude Code Review warnings ([9460d09](https://github.com/aram-devdocs/GoudEngine/commit/9460d09ce49c576f16bc6a63d8aaceec355d3777))
* address code quality review findings in font loader ([ac878c9](https://github.com/aram-devdocs/GoudEngine/commit/ac878c9ea6826bbb9b3112753721ee8c943a49f0))
* **ci:** default empty API responses to 0 in community-stats workflow ([#448](https://github.com/aram-devdocs/GoudEngine/issues/448)) ([e69ada9](https://github.com/aram-devdocs/GoudEngine/commit/e69ada9a85d59f7f1261876bf9aab79b379fc204))
* correct import ordering in core.rs for cargo fmt ([4fb2302](https://github.com/aram-devdocs/GoudEngine/commit/4fb230274966fe6b364601260646efba530046dd))
* **ecs:** forward component_access in Option&lt;Q&gt; for conflict detection ([64350b3](https://github.com/aram-devdocs/GoudEngine/commit/64350b3b40a6efa06a0f8883b54d68e83f31e599))
* **error:** add Input error variants and complete round-trip coverage ([299543e](https://github.com/aram-devdocs/GoudEngine/commit/299543efef55ff0086301c602328b44993503722))
* resolve duplicate load_async when both native and web features enabled ([c517697](https://github.com/aram-devdocs/GoudEngine/commit/c517697cae43d04f1e7fc2e76bb9279ada19d222))
* use create-pull-request in RFC auto-approve workflow ([04b44c5](https://github.com/aram-devdocs/GoudEngine/commit/04b44c5b84d3a00cf01ab6249ac6b8fadbcea09a))


### Refactoring

* extract GoudError methods to separate file and fix formatting ([3f8948e](https://github.com/aram-devdocs/GoudEngine/commit/3f8948edbf40e5cb426d539313f3c898328d4ca5))
* extract sparse set tick tracking to separate file ([e00a53e](https://github.com/aram-devdocs/GoudEngine/commit/e00a53e612a58287ae30a756708961e08e6b5a69))
* **ffi:** consolidate GoudResult/GoudFFIResult and add error query FFI exports ([8fc6442](https://github.com/aram-devdocs/GoudEngine/commit/8fc6442cf8d4ef910ac4d45db1fa8a4b1deda859))

## [0.0.827](https://github.com/aram-devdocs/GoudEngine/compare/v0.0.826...v0.0.827) (2026-03-06)


### Features

* add community stats & download metrics to README ([85537db](https://github.com/aram-devdocs/GoudEngine/commit/85537db9c52a62bd0182cc4429b1f53bb72907b5))
* add hierarchy cascade deletion (despawn_recursive) ([2ec6558](https://github.com/aram-devdocs/GoudEngine/commit/2ec6558c7f2e2c8513ca736c65d1840374af7404)), closes [#193](https://github.com/aram-devdocs/GoudEngine/issues/193)
* add hosted docs site with API reference generation ([2f60b32](https://github.com/aram-devdocs/GoudEngine/commit/2f60b32551db8448a019ed709d82a9904fe41415)), closes [#288](https://github.com/aram-devdocs/GoudEngine/issues/288) [#292](https://github.com/aram-devdocs/GoudEngine/issues/292)
* persist download history and render chart over time ([392e84b](https://github.com/aram-devdocs/GoudEngine/commit/392e84b4990eb94dfa87b0e2abd928cf3d3d3a2e))


### Bug Fixes

* add CopyNativeLib target to all C# examples and update SDK version ([4c0c938](https://github.com/aram-devdocs/GoudEngine/commit/4c0c9389da7d7934f97a2232315384b7b470e9ba))
* address code review — remove dead fields, fix shader_binds, DRY RAF loop ([6e72425](https://github.com/aram-devdocs/GoudEngine/commit/6e72425f70d891155448e65605d1d42826c441ad))
* align OpenGL backend files with main after rebase ([dd5a299](https://github.com/aram-devdocs/GoudEngine/commit/dd5a2994203087e2ccd08f2df6e25bc200ac1bda))
* align workflow output with README placeholder structure ([b3be6aa](https://github.com/aram-devdocs/GoudEngine/commit/b3be6aa0d5b5bc2fc7c9b0048e0680a25f059e6e))
* clean up community stats to show total downloads only ([b8d0b34](https://github.com/aram-devdocs/GoudEngine/commit/b8d0b34d7d404eeb7a6006baedf6d5f0d14bb5fd))
* create api directory before copying rust docs ([32a9726](https://github.com/aram-devdocs/GoudEngine/commit/32a9726a986318e71ff2d306d6925b1afbef36bb))
* deduplicate backend/mod.rs — extract to submodules (898→42 lines) ([40e2ae0](https://github.com/aram-devdocs/GoudEngine/commit/40e2ae05272efbe9358d85f4e7a36a75b374fb53))
* escape hash in C# markdown heading for markdownlint ([9ec8693](https://github.com/aram-devdocs/GoudEngine/commit/9ec8693e80c7680c3571e2a547fbb53b28e5bae5))
* make plan execution protocol tool-agnostic (not Claude Code-specific) ([03f949d](https://github.com/aram-devdocs/GoudEngine/commit/03f949d32216dd207df35dcacc0ed77a535a8d10))
* prefix unread wgpu_backend fields with _ to fix CI dead_code errors ([d521095](https://github.com/aram-devdocs/GoudEngine/commit/d521095f4d3b97e80c1271f3557c7d5a2fb9e5f3))
* remove unused Vec2 import in core/types/tests.rs ([01681b7](https://github.com/aram-devdocs/GoudEngine/commit/01681b7a6cadae8edc0df21c15009d59ac0086a9))
* replace hardcoded path with {{MAIN_REPO_PATH}} placeholder, clarify plan re-interpretation scope ([aef4086](https://github.com/aram-devdocs/GoudEngine/commit/aef4086ea222d9e8ad81a9e857cf829cd48a3776))
* replace RefCell with Mutex for sound Sync impl ([6a3a426](https://github.com/aram-devdocs/GoudEngine/commit/6a3a426a9474ca860439c041296e6ea1747789a0))
* resolve clippy::module_inception in all tests.rs files ([51af6a2](https://github.com/aram-devdocs/GoudEngine/commit/51af6a2ce371efa9a703080bd0cc3c390224403a))
* resolve layer violations and add SAFETY comments ([#175](https://github.com/aram-devdocs/GoudEngine/issues/175), [#177](https://github.com/aram-devdocs/GoudEngine/issues/177), [#196](https://github.com/aram-devdocs/GoudEngine/issues/196)) ([1593d70](https://github.com/aram-devdocs/GoudEngine/commit/1593d70d84a4b107ab93c82f0e517ad5cebbe4f0))
* resolve mdbook download URL in docs workflow ([6b563f6](https://github.com/aram-devdocs/GoudEngine/commit/6b563f664d79876280055f3353fc5ef775c66924))
* resolve remaining clippy errors from test module restructuring ([08842cd](https://github.com/aram-devdocs/GoudEngine/commit/08842cd95b0494ea6eca8e7de42756ba9b1cf3fc))
* resolve WASM input buffering, texture borrow crash, and TS SDK polish ([d942b93](https://github.com/aram-devdocs/GoudEngine/commit/d942b935c9777c9413d90ab04976549d55133376)), closes [#272](https://github.com/aram-devdocs/GoudEngine/issues/272) [#274](https://github.com/aram-devdocs/GoudEngine/issues/274) [#285](https://github.com/aram-devdocs/GoudEngine/issues/285)
* seed history.json, fix PyPI badge and JSONPath query ([7f72d00](https://github.com/aram-devdocs/GoudEngine/commit/7f72d00c7a34ff8f4cda6c73ebf1640829c66449))
* use heading instead of bold emphasis for Downloads section ([475cf90](https://github.com/aram-devdocs/GoudEngine/commit/475cf90ea53395c4d13ec10228e49c1214e89667))


### Refactoring

* add uniform location caching and debug GL error checking ([aab354d](https://github.com/aram-devdocs/GoudEngine/commit/aab354dbb7a6eb4b3a3975b99ed714094e2b1bef))
* remove #[allow(dead_code)] from production code ([597afce](https://github.com/aram-devdocs/GoudEngine/commit/597afcefac72838b6f759fab604ba305aafb5f91)), closes [#214](https://github.com/aram-devdocs/GoudEngine/issues/214)
* remove duplicate restitution/friction from RigidBody ([1a0d818](https://github.com/aram-devdocs/GoudEngine/commit/1a0d8189bd4fe65fc75a18f826cad9d12ec786bb)), closes [#194](https://github.com/aram-devdocs/GoudEngine/issues/194)
* split batch 1 — ECS core files under 500-line limit ([4abb8f7](https://github.com/aram-devdocs/GoudEngine/commit/4abb8f76a41525f781e8e3ed17ca4e08bf75163e))
* split batch 2 — ECS component files under 500-line limit ([7b28d8e](https://github.com/aram-devdocs/GoudEngine/commit/7b28d8e0131ec2fd2ac332bae9b7c9f126c058bd))
* split batch 3 — ECS system files under 500-line limit ([8c0b6f9](https://github.com/aram-devdocs/GoudEngine/commit/8c0b6f913554c73ce92155212862f3d534cfb80d))
* split batch 4 — ECS physics/input files under 500-line limit ([813cb66](https://github.com/aram-devdocs/GoudEngine/commit/813cb667a9fbdb17bfcda20d9a78f07c1cc36c6d))
* split batch 5 — core files under 500-line limit ([1fcfe26](https://github.com/aram-devdocs/GoudEngine/commit/1fcfe26ab92ecaf4c52a8e9a9e9af9b22e9d4ab5))
* split batch 6 — asset files under 500-line limit ([be94b8c](https://github.com/aram-devdocs/GoudEngine/commit/be94b8cca570a1c0f5003425ce872b5bc0cd2385))
* split batch 7 — graphics files under 500-line limit ([78fb841](https://github.com/aram-devdocs/GoudEngine/commit/78fb84141f9196ae6a36a1b19b94b41aea66a869))
* split batch 8 — FFI files under 500-line limit ([0bfc9ca](https://github.com/aram-devdocs/GoudEngine/commit/0bfc9cacc4677534059fbc22ef3f83914aebd02d))
* split batch 9 — WASM/SDK/macros files under 500-line limit ([86319a9](https://github.com/aram-devdocs/GoudEngine/commit/86319a996efa0114a5e18d4ecd3fd582af9644a7)), closes [#173](https://github.com/aram-devdocs/GoudEngine/issues/173)

## [0.0.826](https://github.com/aram-devdocs/GoudEngine/compare/v0.0.825...v0.0.826) (2026-03-03)


### Features

* add autonomous AI agent infrastructure ([#390](https://github.com/aram-devdocs/GoudEngine/issues/390)) ([89ea5a8](https://github.com/aram-devdocs/GoudEngine/commit/89ea5a84f8e6fab6899a12df699688fd4d2eb6f1))
* add plan-review-execute workflow for agent issues ([69f3990](https://github.com/aram-devdocs/GoudEngine/commit/69f3990040aa817862896560e3b20623d04fa573))
* add plan-review-execute workflow for agent issues ([23907d5](https://github.com/aram-devdocs/GoudEngine/commit/23907d5fa39aba32f1d0725a35cc4094886b81de))
* harden agent workflow with Opus, subagents, review feedback loop ([b00d26a](https://github.com/aram-devdocs/GoudEngine/commit/b00d26a1e05febb75a64f9a46f8c65fb45393cc9))
* set up mdBook for documentation site ([e0217ee](https://github.com/aram-devdocs/GoudEngine/commit/e0217ee508f3cb9f9c9b6dab047ad9f649451a0b))
* set up mdBook for documentation site ([84b8337](https://github.com/aram-devdocs/GoudEngine/commit/84b833791220bf0bd82840dcc5d717791866a1e1)), closes [#151](https://github.com/aram-devdocs/GoudEngine/issues/151)


### Bug Fixes

* add --output-dir to napi v3 build commands ([ae6f3c8](https://github.com/aram-devdocs/GoudEngine/commit/ae6f3c8d342d0b8b92ddbcb64662d58229e86480))
* add claude[bot] to allowed_bots in code review workflow ([c0a8edd](https://github.com/aram-devdocs/GoudEngine/commit/c0a8edd48aed7381fc6f5bfb33c9936ba1866015))
* add goudengine-agent to allowed_bots for claude-code-action ([026934a](https://github.com/aram-devdocs/GoudEngine/commit/026934a0feb593bb16b2b043ee05879b0bbb9526))
* address review warnings in schedule module split ([6baa195](https://github.com/aram-devdocs/GoudEngine/commit/6baa1954544253edf6a4ddf70b4e6a9d94bac813))
* address review warnings in world module split ([9cadb81](https://github.com/aram-devdocs/GoudEngine/commit/9cadb810efa2ea8a7d8dbc24d073cdb7e7e25016))
* correct broken links in mdBook documentation ([d58ce54](https://github.com/aram-devdocs/GoudEngine/commit/d58ce54e64db9cabebc451ed5d1e750d40bd8f92))
* correct milestone count in triage audit summary (12 → 13) ([318f8a2](https://github.com/aram-devdocs/GoudEngine/commit/318f8a2ad97db8fcfb545b9a2671014ce57a9e82))
* fix agent workflow crash, false-green status, and cascading runs ([4beca94](https://github.com/aram-devdocs/GoudEngine/commit/4beca940a14b512859b83cbdfc7b37361d9b453a))
* improve agent workflow failure handling and feedback loop ([3287f87](https://github.com/aram-devdocs/GoudEngine/commit/3287f87aaaec8891dfe3516d5bcc39218750467f))
* improve agent workflow failure handling and feedback loop ([a64a16e](https://github.com/aram-devdocs/GoudEngine/commit/a64a16ee94d9423d7da5a03c0d8ed9f5664c29e3))
* rename direct_prompt to prompt for claude-code-action v1 ([19a4c9c](https://github.com/aram-devdocs/GoudEngine/commit/19a4c9c6d42a206a2c90c9459c01f1da0df92f37)), closes [#389](https://github.com/aram-devdocs/GoudEngine/issues/389)
* restore id-token permission required by claude-code-action OIDC auth ([3af9fa1](https://github.com/aram-devdocs/GoudEngine/commit/3af9fa11733750a7117e3164fad06f1a7c8820cf))
* revert synchronize trigger from code review workflow ([415cd18](https://github.com/aram-devdocs/GoudEngine/commit/415cd1848d53b25bbd5762fbb8c2f8499bd6f3d6))
* unblock agent workflow with broadened permissions and budget cap ([8e43925](https://github.com/aram-devdocs/GoudEngine/commit/8e43925629e613c374e13a7545964fb8956052ea))
* unblock agent workflow with broadened permissions and budget cap ([a5bf7a2](https://github.com/aram-devdocs/GoudEngine/commit/a5bf7a2faebfb946623bfa3124766d9f809c5dd0))
* use headings instead of bold for markdown lint compliance ([605c3b4](https://github.com/aram-devdocs/GoudEngine/commit/605c3b455ae0cc754b0c6577d2328046130a4360))


### Refactoring

* split handle.rs (3574 lines) into focused handle/ module directory ([5d48292](https://github.com/aram-devdocs/GoudEngine/commit/5d482923d268ea3aab149a5b09d498a718124943)), closes [#171](https://github.com/aram-devdocs/GoudEngine/issues/171)
* split schedule.rs (8301 lines) into focused module directory ([180c073](https://github.com/aram-devdocs/GoudEngine/commit/180c073465a0b44951156bfd6af30f6b11658854))
* split schedule.rs (8301 lines) into focused module directory ([c716fd9](https://github.com/aram-devdocs/GoudEngine/commit/c716fd97f8b4efe103c2ac1fc480f721dea1e4e3)), closes [#162](https://github.com/aram-devdocs/GoudEngine/issues/162)
* split world.rs (4374 lines) into world/ directory module ([c0f9f85](https://github.com/aram-devdocs/GoudEngine/commit/c0f9f854b51f45b13211dd03994355a9cfa58e90))
* split world.rs (4374 lines) into world/ directory module ([0f9ecb0](https://github.com/aram-devdocs/GoudEngine/commit/0f9ecb08a8a14a2ffe956aec85424fd8fca9d956)), closes [#163](https://github.com/aram-devdocs/GoudEngine/issues/163)

## [0.0.825](https://github.com/aram-devdocs/GoudEngine/compare/v0.0.824...v0.0.825) (2026-03-01)


### Bug Fixes

* add concurrency group and guard workflow_dispatch publish ([#109](https://github.com/aram-devdocs/GoudEngine/issues/109)) ([0c95ca6](https://github.com/aram-devdocs/GoudEngine/commit/0c95ca6b27df9f20f74b34e7e1fe4204da3f0d9b))

## [0.0.824](https://github.com/aram-devdocs/GoudEngine/compare/v0.0.823...v0.0.824) (2026-03-01)


### Bug Fixes

* handle already-published crates in release workflow ([#107](https://github.com/aram-devdocs/GoudEngine/issues/107)) ([cd2e6c1](https://github.com/aram-devdocs/GoudEngine/commit/cd2e6c1042a9180a0a28731dd2b825ea42fd9c39))

## [0.0.823](https://github.com/aram-devdocs/GoudEngine/compare/v0.0.822...v0.0.823) (2026-03-01)


### Bug Fixes

* allow workflow_dispatch to trigger full publish pipeline ([#105](https://github.com/aram-devdocs/GoudEngine/issues/105)) ([d14e383](https://github.com/aram-devdocs/GoudEngine/commit/d14e3834cfef3e729d8e252bd513dece81b0fb68))

## [0.0.822](https://github.com/aram-devdocs/GoudEngine/compare/v0.0.821...v0.0.822) (2026-03-01)


### Bug Fixes

* publish goud_engine_macros to crates.io and auto-merge release PRs ([#102](https://github.com/aram-devdocs/GoudEngine/issues/102)) ([ecb0d3f](https://github.com/aram-devdocs/GoudEngine/commit/ecb0d3f9ddd7cdcc7b9360d068f1cb4510e24700))

## [0.0.821](https://github.com/aram-devdocs/GoudEngine/compare/v0.0.820...v0.0.821) (2026-03-01)


### Features

* SDK developer experience overhaul ([#99](https://github.com/aram-devdocs/GoudEngine/issues/99)) ([a9c3ccf](https://github.com/aram-devdocs/GoudEngine/commit/a9c3ccf31e57809ac81be17e47f732146e6f074b))

## [0.0.820](https://github.com/aram-devdocs/GoudEngine/compare/v0.0.819...v0.0.820) (2026-03-01)


### Bug Fixes

* skip prepublishOnly during CI npm publish ([0fe80a8](https://github.com/aram-devdocs/GoudEngine/commit/0fe80a8582c068dae0ba46a3b227d2968602cbf9))

## [0.0.819](https://github.com/aram-devdocs/GoudEngine/compare/v0.0.818...v0.0.819) (2026-03-01)


### Bug Fixes

* add codegen + build steps to npm and NuGet publish jobs ([#95](https://github.com/aram-devdocs/GoudEngine/issues/95)) ([640a3bb](https://github.com/aram-devdocs/GoudEngine/commit/640a3bbdb09df47918ccd63ee2f75a3bd22d8750))

## [0.0.818](https://github.com/aram-devdocs/GoudEngine/compare/v0.0.817...v0.0.818) (2026-03-01)


### Bug Fixes

* resolve all release workflow build failures ([#93](https://github.com/aram-devdocs/GoudEngine/issues/93)) ([96ca555](https://github.com/aram-devdocs/GoudEngine/commit/96ca5552d83551539c80b07a2ca805687bb0a2bf))

## [0.0.817](https://github.com/aram-devdocs/GoudEngine/compare/v0.0.816...v0.0.817) (2026-03-01)


### Bug Fixes

* auto-merge release-please PR to complete release flow ([#90](https://github.com/aram-devdocs/GoudEngine/issues/90)) ([e16e2a7](https://github.com/aram-devdocs/GoudEngine/commit/e16e2a78405662cfb51b462369fc06bdf4f88221))
* sync crate versions to 0.0.816 and fix release-please updaters ([#92](https://github.com/aram-devdocs/GoudEngine/issues/92)) ([e6cca3f](https://github.com/aram-devdocs/GoudEngine/commit/e6cca3f9ba161cfae9c25492f1333df0c9ae7425))

## [0.0.816](https://github.com/aram-devdocs/GoudEngine/compare/v0.0.815...v0.0.816) (2026-03-01)


### Features

* add async asset loading path for web (Phase 2d) ([184216f](https://github.com/aram-devdocs/GoudEngine/commit/184216f10985ac297e851befa1304feca03b727c))
* add comprehensive AI agent DX infrastructure ([db7b883](https://github.com/aram-devdocs/GoudEngine/commit/db7b88325506d83bd88140e7407b37865586b612))
* add comprehensive AI agent DX infrastructure ([e2277f6](https://github.com/aram-devdocs/GoudEngine/commit/e2277f63cae57d132323d938b1e586cef02842e7))
* add initial graphics and logger modules, restructure rendering components ([5d9e3f3](https://github.com/aram-devdocs/GoudEngine/commit/5d9e3f34c1120b70d99df1efc1b25b7f726baf02))
* add Node.js TypeScript SDK with napi-rs (Phase 1) ([076a0c6](https://github.com/aram-devdocs/GoudEngine/commit/076a0c6e765266dc60d0deba6d02055fcaf30eec))
* add PlatformBackend trait and GlfwPlatform implementation (Phase 2b) ([2c1bc0c](https://github.com/aram-devdocs/GoudEngine/commit/2c1bc0c789884503e8f9f0ccd38d753764171331))
* Add RendererType to support dynamic 2D and 3D rendering ([ccf3929](https://github.com/aram-devdocs/GoudEngine/commit/ccf3929218e1de917b6e26ab2bb9cf98ad6723eb))
* add three-tier agent governance with hook-enforced delegation ([a5e0010](https://github.com/aram-devdocs/GoudEngine/commit/a5e00103fd3db5727e3d61eb0467f4eaae3f99e4))
* add wgpu render backend, wasm bindings, and web TypeScript SDK (Phases 3-4) ([7c9820b](https://github.com/aram-devdocs/GoudEngine/commit/7c9820b726e41e5e883446841f0c09dd0ac6b07f))
* architecture hardening — layer enforcement, circular dep fix, security fixes, SDK split ([a4b2f6e](https://github.com/aram-devdocs/GoudEngine/commit/a4b2f6eac3253f010e608592400cff27cabea45a))
* complete ECS and render backend refactor ([2703f62](https://github.com/aram-devdocs/GoudEngine/commit/2703f62d0deee9bb66d719b45501d9cf3d850807))
* **engine:** Complete ECS and Render Backend Refactor ([f41b634](https://github.com/aram-devdocs/GoudEngine/commit/f41b63480bdf4c4951dca51ebaabc0880323a277))
* feature-flag native platform deps behind 'native' feature ([2b2f587](https://github.com/aram-devdocs/GoudEngine/commit/2b2f58795b3e2d3d500e8de691d4cdc6d4d690ac))
* **ffi:** add Window, Renderer, Input, Collision FFI + C# SDK wrappers ([6fbf87c](https://github.com/aram-devdocs/GoudEngine/commit/6fbf87c5c377c964e48f40a1c2ad428af998008c))
* Phase 1 — 100% FFI coverage (191/191 functions mapped) ([72737ff](https://github.com/aram-devdocs/GoudEngine/commit/72737ffff51d22f842f2fb88bce4f9c11faac97f))
* Phase 2 — working component operations in C# and Python SDKs ([eea73ba](https://github.com/aram-devdocs/GoudEngine/commit/eea73ba21a2f1ccb26d62df0cf81799f1e531cec))
* Phase 3+6 — TS Node FFI-only rewrite, pipeline validation, delegation fix ([43208a1](https://github.com/aram-devdocs/GoudEngine/commit/43208a17ac5e1e4cde9552200a4ac30a891c9afc))
* Phase 5 — create Rust SDK re-export package ([dff4126](https://github.com/aram-devdocs/GoudEngine/commit/dff4126477fad89d3b296d073b26cc2f8e404b21))
* reorganize examples by SDK, add 3D renderer with fog/grid, Python SDK ([4d1575c](https://github.com/aram-devdocs/GoudEngine/commit/4d1575c5d0feeeb91dea1eae1b6a6ee0d75c60ea))
* SDK codegen migration — generate all SDK code from schema ([c5e98bb](https://github.com/aram-devdocs/GoudEngine/commit/c5e98bbb6c541591abe36e3e75fd7abc9011d87c))
* TypeScript SDK, codegen system, and SDK migration ([db017e6](https://github.com/aram-devdocs/GoudEngine/commit/db017e6d8d768e0e20b953fc5afeae5878c78bbf))


### Bug Fixes

* add goud_engine_macros to workspace and dev-dependencies ([4f2802b](https://github.com/aram-devdocs/GoudEngine/commit/4f2802b06c6d134bbe0baad85d2a6ac229ebfb58))
* add MIT license to lint-layers tool ([fc49d69](https://github.com/aram-devdocs/GoudEngine/commit/fc49d6967ad971e91e539b902505e22634a1a48f))
* add title, updateFrame, frameCount, totalTime to Node napi binding ([81a5b1d](https://github.com/aram-devdocs/GoudEngine/commit/81a5b1d5c96f068891ca7281ef871e11745c3b24))
* bump version to 0.0.813 and fix CI for napi/wgpu workspace ([600c55d](https://github.com/aram-devdocs/GoudEngine/commit/600c55d3bbd82d7de5d2c7153bd6ad51575c1fd9))
* C# codegen — use correct types and fields for cached properties ([b3b6670](https://github.com/aram-devdocs/GoudEngine/commit/b3b667058c7181a1081e939b84cf3274eedb42f2))
* check formatting only on committed code, format generated napi files ([aa81879](https://github.com/aram-devdocs/GoudEngine/commit/aa81879d394c9fa0578d9f000f35ed87b227d8ef))
* CI issues — Python Sprite API, wasm codegen, TS native display ([b56d663](https://github.com/aram-devdocs/GoudEngine/commit/b56d663f9f654f4824647fca9b5a61e318eb3eda))
* CI wasm verify paths, add codegen to integration build ([feeda9b](https://github.com/aram-devdocs/GoudEngine/commit/feeda9b0d9e5386c46405cc0bfad30a982f42925))
* **ci:** address multiple GitHub Actions failures ([5217977](https://github.com/aram-devdocs/GoudEngine/commit/52179771f19cec083da338a80dcf5d264d6fb824))
* **ci:** expand markdownlint config to disable failing rules ([a8a66f1](https://github.com/aram-devdocs/GoudEngine/commit/a8a66f14171796ea2d3bb8a63c16e1236bca47f7))
* **ci:** fix .NET and Python SDK CI failures ([a2ed59b](https://github.com/aram-devdocs/GoudEngine/commit/a2ed59b885864775a5307b373c47eed5759848ba))
* **ci:** remove target-cpu=native to fix SIGILL errors ([304c1e7](https://github.com/aram-devdocs/GoudEngine/commit/304c1e785ac8df472f3227c2d6f273c71c04b4d5))
* **ci:** rename Windows DLL to match expected libgoud_engine name ([f35571b](https://github.com/aram-devdocs/GoudEngine/commit/f35571b9346ee99036e8b267e115e01f546cc7b0))
* downgrade version number to 0.0.644 in GoudEngine project file ([150b0d9](https://github.com/aram-devdocs/GoudEngine/commit/150b0d9f2302bc8a218f687ddfd9ef2c7faa184a))
* escape asterisks in README.md for proper formatting ([5468ec6](https://github.com/aram-devdocs/GoudEngine/commit/5468ec6abe6d46e81c25c343bf35c6b1d16dda88))
* gate wasm module on wasm32 target and update wgpu 28 API ([4c2de2b](https://github.com/aram-devdocs/GoudEngine/commit/4c2de2b82b6ea00e3b5823bad37d0947852336e8))
* generate napi-rs Rust via codegen, convert examples to .ts, fix CI ([3b68b70](https://github.com/aram-devdocs/GoudEngine/commit/3b68b701f132e2f57080d20f758d02f1998f3691))
* improve WSL/Linux compatibility for dev scripts ([ff2b12f](https://github.com/aram-devdocs/GoudEngine/commit/ff2b12fe9eeb510e1be33f6165a2b55e7f94954b))
* Python SDK codegen — add bodies for title, totalTime, frameCount properties ([8839c4d](https://github.com/aram-devdocs/GoudEngine/commit/8839c4d3a81a7fec5279077791e52935b5cdc849))
* remove ghost updateFrame from schema, add cargo fmt to CI drift check ([a2adb5d](https://github.com/aram-devdocs/GoudEngine/commit/a2adb5d505a29edc41f215b36897229750642fcb))
* resolve clippy warnings and formatting for CI ([e603655](https://github.com/aram-devdocs/GoudEngine/commit/e6036551e975e2695bbb55510dc544180dcbf634))
* resolve DRY/SSOT/drift issues, harden CI, update all documentation ([95be1db](https://github.com/aram-devdocs/GoudEngine/commit/95be1db91a441ad96dd031ed4c12cf21b268272e))
* resolve release pipeline CI failures ([1c06e39](https://github.com/aram-devdocs/GoudEngine/commit/1c06e39f79d8bead19af1fe89b9d43e6e5e01e37))
* resolve release pipeline CI failures ([49594bd](https://github.com/aram-devdocs/GoudEngine/commit/49594bd5d3eb64ccf331350df38eb0da6ccb9a01))
* resolve TS SDK codegen errors causing 4 CI failures ([8ffd231](https://github.com/aram-devdocs/GoudEngine/commit/8ffd231c1caa91df2315b6c6e53eab9c80982ee7))
* run codegen before formatting check in CI ([7ab7b0c](https://github.com/aram-devdocs/GoudEngine/commit/7ab7b0c0b3013ea4298c3d1aa396655eb5d379b1))
* **sdk:** exclude tests for excluded SDK types ([a2d651a](https://github.com/aram-devdocs/GoudEngine/commit/a2d651a7a77dd570f92cb313c3e5a05f4e33b66f))
* **sdk:** resolve .NET SDK CI build failures ([daab6da](https://github.com/aram-devdocs/GoudEngine/commit/daab6daf895d1816b3b00fb513efd243dfc6b2c6))
* update C# and Python codegen to include shaderBinds in RenderStats ([e415319](https://github.com/aram-devdocs/GoudEngine/commit/e415319650fd510bd65a056ef2ae3e97806a6475))
* update GoudEngine version to 0.0.648 and improve local package restoration ([511186b](https://github.com/aram-devdocs/GoudEngine/commit/511186b5edd59dca91c6da613dc70b9783fe603a))
* update relative paths for texture and map loading in GameManager ([8a8f4d8](https://github.com/aram-devdocs/GoudEngine/commit/8a8f4d8db9f2619900d52087cb2337ba57751811))
* web Flappy Bird rendering — sprite center convention, clear color, dt cap ([7e98897](https://github.com/aram-devdocs/GoudEngine/commit/7e98897876c85f6b679572871c9e7402a04167a8))


### Refactoring

* clean up unused imports and commented-out code, reorganize module declarations ([4251cf5](https://github.com/aram-devdocs/GoudEngine/commit/4251cf52e07781a744ce49e6fa6b8ca4066d4df7))
* clean up unused rotation and scaling code, update README for CsBindGen usage ([f9da176](https://github.com/aram-devdocs/GoudEngine/commit/f9da176247318e7979dc04bab2fcdf84d7aae873))
* comment out tests in TiledManager for future cleanup ([58a3e87](https://github.com/aram-devdocs/GoudEngine/commit/58a3e87432ed1cdd3028d8dccebd829ca018cd6e))
* Phase 0 — dead code cleanup, SDK directory rename, version sync ([970bc7a](https://github.com/aram-devdocs/GoudEngine/commit/970bc7af093a7b8b29785d663a7ec6818c64b0b3))
* remove old shader files and update ShaderProgram initializa… ([455e248](https://github.com/aram-devdocs/GoudEngine/commit/455e2484d69b850859aeb99d9310952d3438b2ae))
* remove old shader files and update ShaderProgram initialization ([a16138b](https://github.com/aram-devdocs/GoudEngine/commit/a16138b022665ec617404409c9936b797ae98b69))
* route renderer3d through RenderBackend trait (Phase 2c) ([47c6bc7](https://github.com/aram-devdocs/GoudEngine/commit/47c6bc708cd9d6541d9e4d8ed9fc501724d194fb))
* split 3 oversized files into directory modules (&lt;500 lines each) ([b56868a](https://github.com/aram-devdocs/GoudEngine/commit/b56868a7df527490938919bf73049dce0c1aaf06))
* update crate-type in Cargo.toml and make sdk and types modules public; clean up logger test output and remove commented code from window module ([96f515a](https://github.com/aram-devdocs/GoudEngine/commit/96f515a4ea522fc548850db2282d9120874e6370))
