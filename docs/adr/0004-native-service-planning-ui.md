---
status: accepted
---

# Use a native service for the planning UI

On 2026-11-09 the user selected a native Rust service with Axum/Tokio and a Vue 3/TypeScript/Vite browser application communicating over same-origin HTTP/JSON. This supports the intended service-based evolution and uses the bounded integration evidence already established, while browser-local WASM execution remains unverified. The registry and non-Send search session remain on one native owner thread; the browser saves its workspace locally and receives incremental results under the [accepted planning contract](../planning/service-contract.md).

This is an architecture decision for the upcoming implementation specification, not a production implementation or deployment.
