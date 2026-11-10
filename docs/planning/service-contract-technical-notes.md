# Service contract: technical fact check

Planning research, checked 9 November 2026. No production implementation, dependency selection lock, new prototype or public-library API change is included.

## Existing evidence

[Architecture](../architecture.md) keeps domain logic independent of UI, networking and persistence. The [service proof](../../experiments/service-proof/README.md) demonstrates a registry and non-Send solver retained on one native owner thread, with HTTP-side commands and control flags. Its bounded observations establish feasibility for those cases; they establish neither production capacity nor a responsiveness budget.

## Browser stack and deployment

**Fact:** Vue's official tooling supports a Vite-powered TypeScript project. Vite produces a production bundle suitable for static hosting. TypeScript transpilation in this toolchain does not itself perform type checking; Vue documents a separate type-checking step. [Vue TypeScript guide](https://vuejs.org/guide/typescript/overview), [Vite production builds](https://vite.dev/guide/build).

**Recommendation:** Vue 3 + TypeScript + Vite is a compatible minimal browser SPA choice for C2. This is a maintainability choice, with no measured speed or size advantage claimed. Serve the static application and `/api/` under the same scheme, host and port. Those components determine an ordinary tuple origin; different paths alone do not create different origins. This is a deployment recommendation, not evidence for a specific hosting implementation. [HTML origin definition](https://html.spec.whatwg.org/multipage/browsers.html#origin).

## Rust HTTP boundary and solver ownership

**Fact:** Axum provides typed JSON extraction and JSON responses. Its state extractor requires outer state to satisfy `Send + Sync`; handler futures must be `Send`. JSON extraction rejects malformed or incompatible payloads, but application validation and the public error body remain contract responsibilities. [Axum JSON](https://docs.rs/axum/latest/axum/struct.Json.html), [Axum state](https://docs.rs/axum/latest/axum/extract/struct.State.html), [Axum handlers](https://docs.rs/axum/latest/axum/handler/index.html).

**Fact:** `std::thread::spawn` requires the moved closure and its return value to be `Send + 'static`. It does not require every value created and used inside that closure to be `Send`. Tokio advises dedicated threads for persistent blocking work; substantial computation inside an asynchronous future can prevent other futures from progressing. [Rust thread spawning](https://doc.rust-lang.org/std/thread/fn.spawn.html), [Tokio blocking guidance](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html).

**Inference/recommendation:** Axum + Tokio is compatible with the proven ownership approach if shared handler state contains only thread-safe handles, control flags and transport snapshots. Construct the registry and borrowing session inside the dedicated owner thread and destroy them there. Commands and replies crossing the boundary must satisfy the selected channel's bounds. An `Arc` or mutex does not automatically make a non-Send solver suitable for shared Axum state. This fact check does not compile a new adapter or prove production shutdown behavior.

## HTTP semantics

These are protocol facts; the endpoint mapping remains a planning decision:

| Status | Meaning and constraint |
| --- | --- |
| 202 | Processing has been accepted and remains incomplete. It promises no eventual success. The response should describe current status and identify a status monitor. |
| 409 | The requested action conflicts with the resource's current state. |
| 422 | Content type and syntax are understood, but the contained instructions cannot be processed. Distinguish this from malformed JSON. |
| 429 | The user has sent too many requests in a period. RFC 6585 permits a `Retry-After` header. |
| 503 | Temporary overload or maintenance prevents service. RFC 9110 permits `Retry-After`. |

[RFC 9110 §§15.3.3, 15.5.10, 15.5.21, 15.6.4](https://www.rfc-editor.org/rfc/rfc9110.html#name-status-codes), [RFC 6585 §4](https://www.rfc-editor.org/rfc/rfc6585.html#section-4).

**Material qualification:** An undifferentiated global owner-capacity limit is not automatically request-rate limiting. Prefer 503 for temporary service overload; reserve 429 for an explicit client admission/rate policy. `Retry-After` accepts either an HTTP date or a nonnegative integer number of seconds. Selecting it as a required API header is stricter than the RFC's optional use. [RFC 9110 §10.2.3](https://www.rfc-editor.org/rfc/rfc9110.html#name-retry-after).

## Browser workspace persistence

**Fact:** Web Storage exposes synchronous string reads and writes. Access to `localStorage` can throw `SecurityError`; writes can throw `QuotaExceededError`. Storage is associated with the origin, user agents may remove it, and the standard advises authors to assume no locking between browser contexts. [HTML Web Storage](https://html.spec.whatwg.org/multipage/webstorage.html).

**Recommendation:** Use it only for a small, versioned browser workspace snapshot with guarded parsing and explicit save-failure handling. Keep language preference separate from domain request identity so changing language does not imply a new computation. That separation is a product/data-contract choice, not a Web Storage requirement. Local persistence does not preserve a native solver or establish restart recovery. No payload-size threshold or write-frequency performance claim is established here.
