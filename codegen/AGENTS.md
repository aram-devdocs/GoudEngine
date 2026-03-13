# codegen/ -- SDK Code Generation

- Schema changes here affect multiple SDKs at once.
- Treat `codegen/goud_sdk.schema.json` as the public contract source.
- Prefer template or generator fixes over hand-editing generated SDK output.
- After generator changes, run the narrowest verification that proves the affected SDK surfaces still build.
