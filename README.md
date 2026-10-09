# File authority IPC contract

Version 0.1.15 replaces this crate's exact Univers dependency pins with bounded
compatible ranges, retaining the accepted lower bounds and the workspace's
contracts-world rc.15 lock. All 0.1.14 APIs, schemas, codecs, features and bounds
remain unchanged. This crate has no direct contracts-world requirement; WorldIPC
and Files Policy supply that dependency. A consumer can select published
WorldIPC 0.4.0-dev.5, Files Policy 0.1.0-dev.7 and contracts-world 1.0.0-rc.18
without requiring this workspace to select rc.18 before publication.

The WorldIPC range names both prerelease cores, from 0.3.12-dev.12 through
0.4.0-dev.5 inclusive. Cargo does not admit a new core's prerelease through a
plain stable upper bound such as `<0.5.0`; the inclusive upper bound admits the
verified 0.4.0 candidate while preserving the existing 0.3.12 workspace lock.
Later WorldIPC candidates require an explicit compatibility review. Other
Univers ranges follow the workspace dependency policy with accepted prerelease
lower bounds and the next minor upper bound. The downstream compatibility check
enables every FileIPC feature together with published contracts-world rc.18,
WorldIPC 0.4.0-dev.5, forwarded-context 0.1.3-dev.6, AuthIPC 0.2.1-dev.3 and
Files Policy 0.1.0-dev.7. These checks establish Rust graph/API compatibility;
consumers retain the authority and recovery obligations described below.

Version 0.1.14 admits issued WorldIPC0.3.12-dev.12 and Files
Policy0.1.0-dev.6 for the C1rc15/C3dev12 World SDK graph while preserving every
0.1.13 capability, retention, world-verifier and world-policy-projection API,
schema, operation, codec, feature and bound. It does not add feature-off
behavior, copied guards or local authority; consumers still prove real
World/File policy, current Auth and actual same-store records through their
owner graphs.

Version 0.1.13 admits actually issued Files Policy0.1.0-dev.5 for the optional
`world-policy-projection` graph. It retains the issued WorldIPC0.3.12-dev.10
pin and every 0.1.12 schema, operation, identity/fingerprint, codec, feature and
bound. The new dependency supplies Files-owned trusted-local original-admission
observation and terminal reconciliation, including pre-acquire originals. This
IPC package adds no recovery wire or authority: its projection stays readonly
historical data. Genuine original Access, current terminal authorization/drain
and World same-store predicate revalidation remain consumer obligations.
Publication requires the normal owner gates/hooks, clean origin/develop source,
package verification and Registry-only recovery/codec consumer acceptance.

Version 0.1.12 admits actually issued WorldIPC0.3.12-dev.10 for the selected
ordinary Subject-policy session contract graph. Every 0.1.11 File/Content API,
wire encoding, feature, default and bound stays unchanged; downstream consumers
explicitly enable WorldIPC `subject-policy-session` when required. Issued World
SDK0.2.1-dev.8 still pins WorldIPCdev8 and requires its own owner successor.
Session carriers remain data; physical World predicates/current Host Auth and
Files public Access/original continuation/startup/ACL remain separate prerequisites.

Version 0.1.11 admits issued WorldIPC0.3.12-dev.8 for the combined Files-policy,
Subject-disposition and opt-in ordinary Subject-policy projection contract graph.
It preserves every 0.1.10 API, feature, schema, carrier, encoding and bound; downstream
consumers enable WorldIPC `subject-policy-projection` explicitly when required.
Issued WorldSDK0.2.1-dev.7 still fixes WorldIPCdev6 and needs its own owner successor
before a unified SDK graph is possible. The Files-owned complete UNTRUSTED Access
transport primitive and native original continuation remain separate prerequisites.

Version 0.1.10 admits issued WorldIPC0.3.12-dev.6 for a coherent combined
Subject-disposition/Files-policy graph while retaining Policy0.1.0-dev.4.
Every 0.1.9 carrier, schema, bound, encoding and public feature stays unchanged.

Version 0.1.9 binds `world-policy-projection` to issued Files Policy0.1.0-dev.4,
which accepts canonical bare descriptor SHA256 alongside unchanged prefixed proofs.
All 0.1.8 schemas, operations, encodings, bounds and features remain unchanged.

Version 0.1.8 adds opt-in `world-policy-projection`: complete untrusted same-store
Files held snapshot projection, using the issued Files decoder and bounded whole
messages. See [CONTENT_WORLD_POLICY_PROJECTION.md](CONTENT_WORLD_POLICY_PROJECTION.md).
All 0.1.7 encodings and bounds remain unchanged.

Version 0.1.7 adds mandatory separate current readonly observer operations for saved
original and terminal verification under `world-verifier`. Original historical
proofs and all 0.1.6 encodings remain unchanged; current Auth stays World-owned.

Version 0.1.6 adds opt-in `world-verifier` for the explicit reverse Content-to-World
original reservation/journal verification callback. Full requests/observers/terminal
records and bounded opaque witnesses remain untrusted wire values; see
[CONTENT_WORLD_VERIFIER.md](CONTENT_WORLD_VERIFIER.md). Existing Content defaults
and encodings remain unchanged. World SDK dev.3 requires an owner dependency update.

Version 0.1.6-dev.1 adds opt-in `retention` public Content acquire/original
restore/pureRead status/owner drain/terminal and accepted-reference lifecycle
envelopes. See [CONTENT_RETENTION.md](CONTENT_RETENTION.md) for authority,
composition and recovery obligations. It requires published Data =rc.3 and pins
WorldIPC dev.4/C1 rc.11/codec dev.3. Wire validation grants no backend or business
acceptance. Existing default-feature variants retain their bytes.


Version 0.1.5 adds optional `capability` wire values for the selected-World
`/v1/file/capability` endpoint. Request variant ordering, legacy decode-only
variants, reply payloads, the World envelope and request bound remain unchanged.
This feature contains no File service, transport, authentication or persistence.

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

The owner uses Rust/Cargo 1.89 or newer, as declared by rust-version. The
`release-workbench` profile inherits the normal release library profile; it
publishes no native binary or runtime pointer.

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

Version 0.1.3 appends four World-owned catalog operations. Path-only requests
reject caller-supplied scope, timestamps, records and preconditions:

- `EnsureDirectory` returns the authoritative directory record and `created`;
  an existing directory preserves its record, while an existing file conflicts.
- `RemoveEmptyDirectory` returns `removed`; a missing path is false, while a
  file or a nonempty directory conflicts. World checks descendants and deletion
  against the same catalog revision.
- `DeleteFile` returns `deleted`; a missing path is false and a directory conflicts.
- `MoveFileRecord` accepts `sourcePath` and `targetPath`, returning `record` and
  `changed`. Missing source returns None/false, including equal paths. Equal paths
  with an existing file return its original record/false without changing time.
  A real move returns the World-generated target record/true. A source directory
  or descendants, occupied different target, or concurrent revision conflicts.

World owns path/tenant validation, time and atomic revision CAS. Invalid scope
or path returns Validation; other errors retain FileAuthorityError. These
results describe this call's linearized outcome, not a durable idempotency or
historical receipt. Do not retry a possibly executed deletion/move automatically
or infer original success from a later equal/missing read after a lost reply.
Business Move/Trash/Restore/Purge continue to use the existing namespace workflows;
the simple move supports their individual sidecar/version repair steps without
nesting another workflow. Byte transfer stays on ContentStore. No implementation
or new SDK is included in these envelopes.

Version 0.1.4 adds `ReconcilePendingContentWrites { mode, limit }` and
`ContentWritesReconciled`, reusing C0 Data's FileContentReconciliationMode and
FileContentReconciliationReport. World checks real Content state and owns
Finalize/Abandon/conflict decisions. The envelope preserves the caller's limit;
World validates its existing 1..=10000 bound. Audit does not request repairs.
Unconfigured Content/recovery returns an explicit error, never an empty success.
Reports describe the performed scan, not a durable replay receipt. This adds no
client/server implementation, startup gate or automatic retry/fallback policy.
