# Original Content retention wire

The opt-in `retention` feature adds `content::Request::Retention` and
`content::Reply::Retention` at the end of the existing enums. All existing wire
values keep their encoding. The lane's public capability is
`content-original-retention-v1`; `Probe` must positively advertise it before a
consumer sends another operation. The existing Content request limit remains
unchanged for existing operations. Retention requests use the 8 MiB limit and
complete-body replies use the explicit 136 MiB limit (64 MiB body as bounded named
byte arrays plus metadata). The strict retention decoders require named
MessagePack or JSON and reject duplicates, unknown nested fields, trailing input,
malformed/oversized frames and unsupported original operations. Use `Request::decode_content_named/json` and `Reply::decode_content_named/json`
for the COMPLETE existing Content envelope (or `decode_named/json` for the
extracted lane). Generic `Deserialize` is not the strict ingress decoder.
Complete body bytes are skipped in the second metadata shape pass after typed
decode and size/SHA validation; they are not expanded into JSON nodes.

## Authority and public contract

Files owns byte mutation/exclusion, actual physical root and journal enrollment,
monotonic retention/reference generations and current Files ACL/policy. World
owns its SQLite Artifact descriptor, reservation, catalog, semantic acceptance,
original Data journal and removal of accepted references. Framework supplies
live Auth and authenticated owner channels. This contract owns only public
transport composition; it owns no backend, grant, drain executor or World records.

`Original` retains the complete published WorldIPC spatial/source invocation,
signed original context/action bytes, C1 original commit admission, actual assigned
World runtime, canonical C1 immutable Artifact proof and Data3 Content descriptor.
Its validation checks full original payload correlation, exact proof-byte digests,
credential presence, expiry bounds, organization/World/fence, material hash/size
and the acquisition action. `Acquire` supports original spatial Apply and source
Select; pure reads, previews and recovery cannot acquire. Source material must
match the exact signed source digest/size and the authenticated World material
producer must also bind the supplied Artifact reference to that original source.
No decoded value grants permission or verifies signed proof cryptography.

`PhysicalStore` is a server-produced identity binding to the original concrete
physical root/journal and their incarnation. Its digest covers the owner's FULL
actual identity record (including original canonical paths/device/inode where
supported); IDs are resolved only by trusted mounting, never paths to open. A
`Retained` value includes the full original, physical identity, positive generation
and full original pin/policy-record digest. It is correlation, not a transferable
lease. Every action loads the original journal and compares all these fields.
Missing, corrupt or copied stores/pins remain unavailable and held; they are
never reconstructed from this request body.

`Restore` loads the recorded original and rereads the entire bounded immutable
body under actual current Auth/policy. It must not create another generation or
replace expired proofs. `Status` requires a separate pureRead/readonly canonical
observer invocation at the original scope/fence. Owners authenticate that observer
and check visibility; status never acquires, stages, reconciles or writes a ledger.
Complete bytes returned by Acquire/Restore must match size and SHA256; descriptor,
URL, leading bytes or transport success alone are insufficient.

`Drain` carries the full published WorldIPC original reconciliation request on an
authenticated owner channel. The reply is `DrainPending`, which keeps retention.
`WorldTerminal` composes the FULL published original reconciliation ACK with
COMPLETE original World terminal journal bytes and their SHA256 (bounded to
1 MiB); `Terminal` preserves its runtime,
result/receipt/outcome and no-future-writes assertion covering OWNED AND QUEUED
Data work. The owner verifies real original journal/checkpoint truth and sender
before changing state. Expiry is intentionally not checked for authentic original
management/drain. A caller's renewed expiry/body, cancellation, timeout, PID,
process death, old ACK or an ordinary Terminal reply cannot prove drainage.
Staged/not-reserved ACKs do not release a previously acquired body. An accepted
World result is rejected by Terminal and must use RetainReference instead.

`AcceptedReference` binds a Files reference ID and positive reference generation
to the full original accepted World ACK AND complete original terminal journal. The reference ID is the accepted spatial bundle ID or source selection ID;
Files authenticates that exact binding. `RetainReference` leaves
the body excluded. `ReferenceRemoval` is the Content-owned public removal carrier:
exact recorded accepted reference/ACK/generation, removal checkpoint ID, actual
World sender runtime and COMPLETE raw original World removal journal bytes with
SHA256 (bounded to 1 MiB). The raw journal remains World private storage encoding,
not a replacement/copy of a canonical DTO. World must authenticate these received
bytes, prove actual durable reference removal and no future reattachment against
the recorded original. Files calls `Request::validate_recorded` against its actual stored original and
accepted reference before dispatch; release without that recorded reference is
denied. Shape
validation cannot interpret that journal or prove its truth. This operation adds
no general World remove/unbind action. Without a genuine World removal producer,
ReleaseReference returns unavailable and the body remains pinned.

## Composition and delivery

Dependencies are Registry-only Data =1.0.0-rc.3, C1=1.0.0-rc.11,
WorldIPC=0.3.12-dev.4, codec=0.1.3-dev.3 through WorldIPC. Original signed
SecurityContext includes the published credential/delegation fields; no Auth DTO
or separate proof format is copied. Files' real authorizer must use published
AuthIPC0.2.0/current Framework>=2.5.34 composition. This wire crate does not need
an AuthIPC library dependency to carry the unchanged signed bytes.

Files mounts the actual RetainedObjectStore and ContentEngine::new_retained on
ALL byte/upload/delete/GC/direct writers. Authentic complete old-engine and
pre-enrollment verified/issued/queued writer drain is mandatory before first
enrollment. The old Content main is not retention-ready by consuming this crate.
World mounts the actual SQLite reservation and same operating raw policy/CAS
predicates, repeats real current Auth/policy before and after awaited checks,
and uses its existing final conditional acceptance. The witness and raw backend
are trusted injected ports, never request-body installed authority.

There is no cross-store atomic transaction. World drains its original owned AND
queued work and authenticates the full original result; Files then conditionally
updates only the exact original generation. A lost/uncertain reply remains held;
retry restores or inspects the recorded original. Accepted references remain pinned
until original authenticated removal. Backend error, replaced stores, missing pin,
changed original identity, successor generations and corrupted body stay denied.
Binary rollback does not imply journal rollback or safe data recovery.

Contract tests demonstrate wire preservation, bounded strict decoding and pure
correlation. Publication and Registry-only consumer compilation are producer
evidence only. They prove no real backend enrollment, migration quiescence, World
SQLite reservation, accepted reference removal or installed HVAC Chiller workflow.

Frozen test JSON is public synthetic unsigned C1 rc.11 fixture data from
`spatial/mutation/fixtures/reconciliation-v1.json` and `governed-source-v1.json`;
tests import the published canonical Rust types. Fixture strings, journals and
physical identities are not signing proof or runtime enrollment evidence.
