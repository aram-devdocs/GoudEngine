# Changelog

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
