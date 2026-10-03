# Content-to-World original verifier contract

The optional `world-verifier` feature enables `content_world_verifier` and reuses
`retention`; default features remain empty. This is an independent reverse callback
family. It does not add/relabel a Content Acquire, Content Reply or WorldIPC Evidence
operation. Existing File/Content and 0.1.6 verifier encodings, and immutable 0.1.6-dev.1, are preserved.

Public provider `world`, direction `content_to_world`, capability
`world-content-original-verifier-v1`, schema `univers.world-content-original-verifier/v1`,
endpoint `/internal/v1/world-content-retention/verify`, logical URL
`http://world-content-verifier/internal/v1/world-content-retention/verify`.
Framework trusted selected-World composition resolves that service to the genuine
selected running World listener. The URL is not a new public route or caller-selected
socket. Secrets, issuer, authenticated Content channel and routing configuration are
not request fields and remain Framework composition.

## Authority, operation and response

Files owns actual physical body/pin/policy lifecycle. World owns actual original
SQLite/operating reservation, material/current policies, semantic acceptance,
original flight and owned/queued Data journal. Framework owns original channel/Auth
composition. The contract owns transport shape and correlation only. Every request
and reply remains UNTRUSTED, including `Status::UntrustedWitness`; deserialization
cannot reconstruct a verified principal, MaterialReservation, trusted token or
permission. There is no serialized allow flag and no general remove/unbind operation.

Every Request includes exact schema/provider/capability/direction and one operation:

- `VerifyAcquisition(Original)` carries FULL unchanged canonical invocation, signed
  context/action header strings, original caller/credential/delegation/org/World,
  action/provider/runtime/fence/payload/proof digests/expiry, immutable Artifact and
  Data3 Content material. World loads the real existing reservation/flight and verifies
  independently authenticated selected Content channel, complete original codec proof,
  current Auth/four object policies, actual material/SQLite/operating witnesses and
  issued/queued work before and after awaits. Verification never creates a reservation.
  `validate_acquisition_at` checks only structural exclusive expiry bounds. Runtime
  authentication/current policy/witness truth are additional World obligations.
- `VerifyOriginal(Retained)` carries the exact original, Content physical root/journal
  identity/incarnation, generation and FULL policy/pin record digest. World MUST have
  already durably recorded that returned Content binding in its ACTUAL original
  journal; `validate_recorded` compares that loaded binding, never request-as-record.
  Original expired proofs remain byte-identical. This legacy operation has no
  independently fresh caller credential. A selected Content channel/capability is
  NOT current caller permission: expired original calls MUST stay held/unavailable.
  Use the additive WithObserver operation for independently authorized saved-original
  verification; no resealing/renewal/rebase/new generation occurs.
- `InspectOriginal { retained, observer }` carries a separate FRESH canonical
  pureRead/readonly Invocation alongside the unchanged original. World uses the
  published complete codec and current scoped Auth, matches genuine original caller/
  credential/delegation/org/World/fence/record and checks CURRENT read policies even
  after acceptance. Structural checks do not open the observer proof or verify actor.
  Read revocation denies disclosure. No reserve/restore/drain/ACK/release/stage/
  selection/revision/original journal writes are permitted by this callback.
- `VerifyTerminal { retained, terminal }` carries FULL unchanged WorldTerminal:
  canonical ACK/runtime/admission/result/receipt/outcome plus complete original World
  journal bytes/digest. World verifies actual original storage/journal and durable
  OWNED AND QUEUED no-future-write truth against its loaded terminal. `validate_recorded`
  requires that actual complete terminal, not body-supplied ACK. Unknown/staged/missing/
  corrupt/copied state or generation mismatch remains held; sender identity alone,
  timeout, process/PID death, capability expiry and old ACK cannot release a successor.
  This verifier itself neither drains nor releases. Reference removal is unsupported.

Version 0.1.7 appends two operations without altering any legacy field or encoding:

- `VerifyOriginalWithObserver { retained, observer }` requires a separate complete
  canonical current readonly `Invocation` beside the unchanged saved `Retained`.
- `VerifyTerminalWithObserver { retained, terminal, observer }` adds the same mandatory
  observer to full unchanged `Retained` and `WorldTerminal`. Actual complete recorded
  terminal comparison remains mandatory; verification never drains or releases.

The new observer must be Spatial `GetOrResume(readOnly=true)` or SubjectSource
`SubjectSourceGetOrResume(readOnly=true)`, in the original invocation family, with
exact original operation ID/idempotency key/input request digest, organization,
World and selected ORIGINAL revision fence. Shape validation cannot open opaque
signed headers. World MUST authenticate both fresh headers with the published
codec/current Auth, enforce exclusive fresh proof expiry, and bind original caller,
credential, delegation chain, provider, readonly action, exact observer input and
current callback scope to its genuine selected channel/current authenticated caller.
Changed/revoked/expired/missing observer denies; historical original mutation proofs
remain exact evidence and MUST NOT be passed off as current authorization. No optional
observer or legacy fallback is permitted for these operations. Owner checks occur
within the actual verification, including rechecks after awaits; no cached two-step
allow, body ACK or identity-only allow. `validate_recorded` compares full loaded
original binding and terminal independently of observer authentication.

Reply includes the FULL sent Request on every status, not a reduced digest projection.
`validate_for(sent)` requires exact operation/schema/original/retained/current observer/
terminal correlation. A well-correlated response still requires genuine sender and
World witness truth. Closed statuses are `UntrustedWitness`, `Denied`, `Unavailable`
and `RecoveryRequired`; failure reasons are bounded. There is no success-to-authority
conversion. `FullWitness` carries independent COMPLETE opaque actual reservation/
witness bytes and original journal bytes, each with full SHA256. Private World
storage encoding stays opaque; non-JSON bytes, unsigned max-u64 values and tails are
preserved rather than projected into a guessed semantic DTO. World interprets the
bytes from its actual store; Files receives them only through the genuine channel.

## Bounds, strict encoding and recovery

Frames are <=8 MiB. FULL witness and journal have independent positive <=1 MiB bounds;
WorldTerminal's existing full original journal bound also remains unchanged. Strict
`Request/Reply::decode_named` requires named MessagePack, depth<=64, typed-first duplicate
checks, no trailing bytes, unknown nested-field rejection before discarded canonical
fields can be accepted, and operation/schema checks. `decode_json` enforces the same
bounded full schema/correlation shape with duplicate/trailing denial. Arbitrary direct
Serde Deserialize is not the strict callback boundary. Structured messages have no
byte-buffer exception or caller-installed authority configuration.

World loads original physical storage and existing journal; no body-provided database
is opened. Missing/corrupt/copied originals are unavailable, never adopted/recreated.
Readonly saved-original/terminal verification after original admission expiry requires
an independently fresh WithObserver invocation and real current scoped Auth, not
merely selected owner-channel authority; a fresh acquisition still rejects that expiry.
A readonly observer is independently authenticated and cannot change the original.
Unknown reply/queued-write uncertainty keeps body/World original held. Cross-store
callbacks are ordered validations, never an asserted SQL/KV/Content atomic transaction.
Binary rollback does not roll back World/Content journals or restore authority.

## Delivery and current consumer constraint

Library SemVer 0.1.7 is an additive opt-in successor to immutable 0.1.6, separate from any native/app
stable/development deployment pointer. World listener is mounted only after actual
Registry publication and genuine owner implementation, not from this source package.
All canonical dependencies remain Registry Data=rc.3, Shared=rc.2, C1=rc.11,
WorldIPC=0.3.12-dev.4, codec=0.1.3-dev.3, Operation=rc.5. AuthIPC0.2.0/current
Framework>=2.5.34 remain runtime composition requirements; signed Auth carriers are
unchanged, and this contract does not need another Auth implementation dependency.

Actual Registry `univers-aip-world-client=0.2.1-dev.3` declares
`univers-aip-contracts-file-authority-ipc >=0.1.4, <=0.1.6-dev.1`. That upper bound
rejects both 0.1.6 and 0.1.7. World SDK owner must upgrade its supported authorityIPC bound to the
ACTUALLY published successor, enable `world-verifier` for the callback consumer and
implement the real typed listener/SDK/current Auth/SQL+operating verification. Do not
force the old SDK into a fictional single graph, downgrade, copy types, patch paths or
disable required file-capability/retention features. No new C1/Data/Auth producer is
required by this request.

Tests cover real published olddev1 golden named bytes under default and opt-in,
canonical full request/observer/ACK fidelity, opaque binary and max-u64 preservation,
strict decoding/correlation and explicit recorded-binding comparisons. Their fixtures
are synthetic unsigned values and do NOT establish live channel/Auth/four policies,
actual SQLite/operating witness or genuine terminal quiescence. Local packaging is
not publication or installed Files/World/native/HVAC business acceptance.

World SDK/native owner consumption: handle the two new operation variants with the
existing full typed Request/Reply at the same endpoint. Files must consume a real
World SDK successor supporting ACTUALLY published IPC 0.1.7, with mandatory
ipc-file-capability, retention, ipc-content and world-verifier support in one current
canonical Registry graph. The World owner selects/checks/publishes its SDK version;
this library does not guess that coordinate or edit SDK/native/Files repositories.
Old 0.1.6 decoders reject the new operation tags; preserve explicit unsupported/held
failure rather than retrying legacy operations with expired mutation proof.
