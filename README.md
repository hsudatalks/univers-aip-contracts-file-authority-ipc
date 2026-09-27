# File authority IPC contract

`Request` and `Reply` retain the existing File authority enum representation and
published C0 Data payloads. This package also publishes endpoint/client URI,
capability, the existing 30-second request timeout value, and 2MiB request,
4MiB metadata response and 16MiB collection response bounds. It has no transport,
client, server, retry, storage or quota/grant/recovery implementation.

World owns authoritative persistence, permission and lifecycle decisions. Files
owns its consumer adapter. Both can use these values with generic bounded IPC
without acquiring each other's implementation. Envelopes alone do not authorize
an operation or bind a selected World/tenant. Callers choose transport timeout
policy and servers enforce the relevant protocol bounds. Content bodies use a
separate protocol and are not File-authority metadata messages.

Run `bash scripts/check.sh`; publish a clean committed candidate with
`bash scripts/publish.sh`. All dependencies are Registry packages. C0 Data must
be at least 1.0.0-rc.2; this release does not claim compatibility with rc.1.

Version 0.1.1 adds `content::{Request, Reply}` plus the existing `/v1/content`
path and byte/chunk bounds. These tagged envelopes reuse C0 Data content types
and preserve the Content Engine wire. No client/server, grants, storage, virtual
paths, upload lifecycle or timeout/retry policy is moved into this package.
Files owns byte storage and capability verification; World owns its scoped
consumer and semantic File lifecycle.

Version 0.1.2 adds `Request::PrepareContentWrite` and
`Reply::ContentWritePrepared`. The request declares path, digest, byte size,
Replace/CreateOnly/IfHashMatches condition, and an optional existing C0 Data
mutation link. World validates that link and binds scope, previous catalog
state, intent identity and timestamps through its existing atomic claim.
Those authority fields are not accepted in the preparation request.
`ConditionNotMet` is distinct from a claimed write and from a backend failure;
`Claimed` contains the original C0 Data claim, including retry identity.
No byte-transfer implementation or lifecycle policy lives in this crate.
Existing endpoints and variants keep their encoding; older servers cannot
handle the new variant and callers must not fall back to self-issued authority.
World can expose this operation through a local server facet without changing
or upgrading the C0 Data Port used by existing consumers.
