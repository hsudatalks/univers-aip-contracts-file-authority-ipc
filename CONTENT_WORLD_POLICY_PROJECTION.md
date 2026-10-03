# Complete World Files-policy projection

`world-policy-projection` is optional and implies `world-verifier`/`retention` plus
the issued `univers-file-governed-policy=0.1.0-dev.4` pure snapshot decoder. Defaults
remain empty. Library 0.1.9 preserves every legacy .7/.6 Content/File/verifier wire,
operation and bound. Module `content_world_policy_projection` uses the SAME selected
reverse World endpoint/provider/direction/capability/client URL as `world-verifier`.
The NEW explicit schema is `univers.world-content-files-policy-projection/v1`.
Old .7/.6 decoders reject this schema/envelope; unsupported profile stays held.
There is no legacy fallback returning a missing/hash-only policy projection.

## Authority and canonical request

Files owns policy/exclusion/original-admission row codecs, physical byte pin and
final release. World owns genuine same ORIGINAL operating-store/material pin and
actual full owned-plus-queued terminal journal; Framework owns selected routing and
current authenticated scope. The library owns shape/correlation/Files' pure decoding.
No trusted Access, SelectedStorage, authorizer, reservation or Rust permit is
serialized/reconstructed. An operating store identity claim is never a path/KV API,
permission or database to open. No private World MaterialPin/Files row codec is copied.

`Request { schema_version, provider, capability, direction, verification }` preserves
the COMPLETE canonical legacy `content_world_verifier::Request`. Only these nested
operations are allowed:

- `VerifyAcquisition(Original)`: real genuinely held policy/exclusion/admission rows
  required. Acquisition expiry still uses the unchanged canonical admission bounds.
- `VerifyOriginalWithObserver { retained, observer }`: unchanged full saved original
  plus separate genuinely CURRENT readonly observer; no expired mutation-proof fallback.
- `VerifyTerminalWithObserver { retained, terminal, observer }`: same fresh input and
  exact full canonical terminal ACK/journal, not a body ACK or successor/released rows.

Current observer caller/credential/delegation/action/input/org/World/original fence and
callback scope must be authenticated by World actual codec/current Auth. After every
physical await World rechecks actual same-store original policy/exclusion/admission,
material/pin and current Auth. Terminal additionally proves actual full original ACK
and durable OWNED AND QUEUED no-future-write disposition. Projection is readonly:
no reserve/stage/release/renew/reseal/rebase/revision/journal writes or raw KV queries.

## Complete untrusted projection and public decoder

`Reply { request, status }` echoes the FULL sent Request in every status. `validate_for`
requires exact request/original/retained/current observer/terminal correlation.
Closed status tags are `untrusted_projection`, `denied`, `unavailable`,
`recovery_required`; failure reasons remain independently bounded. Only
`Status::UntrustedProjection(FullPolicyProjection)` carries a complete projection:

- `original_operating_store_identity`: bounded original same physical operating
  store/incarnation identity claim, not Content's separate byte-retention store.
- `files_held_policy_epoch`: positive lossless u64 original epoch.
- `files_held_policy_snapshot`: exact FULL bytes emitted by the issued Files-owned
  `FilesPolicyAuthority::held_policy_snapshot(&actual_original_access)` on that store.
- `files_held_policy_snapshot_digest`: full SHA256 of those exact bytes.
- `witness`: COMPLETE existing `content_world_verifier::FullWitness`, preserving
  opaque full reservation and original journal bytes/digests and independent bounds.

For terminal projection, witness original journal bytes/digest must equal the full
canonical terminal's journal. Missing field/empty snapshot/zero epoch/digest mismatch
fails. Pure shape success does NOT interpret private owner rows. Call the provided
`FullPolicyProjection::validate_policy_for(&actual_selected_storage, &actual_original_access)`
or `Reply::validate_policy_for(&sent, &actual_selected_storage, &actual_original_access)`.
These invoke ONLY the issued public
`univers_file_governed_policy::snapshot::validate_held_policy_snapshot`, compare the
returned epoch and trusted-composition-selected original store, and return u64 data,
never a capability/permission. Reply additionally correlates actual Access's full
canonical fence/material/runtime/principal/credential with the original request.
Actual complete unchanged Access, including its canonical scope, original context,
admission and delegation, MUST come from owner-recorded original composition, not
from reconstructing a trusted object from this response. Files' decoder validates
ALL original raw row bindings, including full Access and explicit ACL/delegation.

The issued snapshot codec is named MessagePack DataRC3 `Vec<KvValueCondition>`, exactly
three ordered FULL policy/exclusion/original-admission rows (ledger raw equals
exclusion raw). No partial/digest-only row substitute or JSON float normalization.
Files alone owns schema/keys/order/private interpretation; this contract does not
copy that codec. A saved valid snapshot can decode after actual release: this is
historical binding data, NOT live permission, reopened pin or a new edit/release.
Actual producer/channel/Auth/current same-store predicates must still be checked.

## Whole-message profiles and recovery

Legacy verifier retains its 8MiB message and independent positive 1MiB witness/
original-journal bounds. New snapshot raw bytes are positive <=8MiB, original operating
store identity <=4096 UTF8 bytes. New complete nested verification must encode within
legacy 8MiB in BOTH JSON and named MessagePack. NEW Request <=8MiB+64KiB (headers);
NEW whole Reply <=64MiB in either codec. All strict decoders reject unknown/nested/
duplicate/trailing/positional malformed fields. Configure transport with these actual
new whole-frame limits; legacy SDK 8MiB response profiles cannot silently truncate.

Plain Vec<u8> JSON arrays use at most 4 bytes per input byte: snapshot<=32MiB plus
witness/journal<=8MiB plus full nested request<=8MiB and bounded headers/identity,
comfortably within64MiB. Named MessagePack arrays use <=2 bytes per input byte:
snapshot<=16MiB plus witness/journal<=4MiB plus full request<=8MiB and headers.
Snapshots themselves already use Files' independent 8MiB encoded bound. No hashing,
projection, truncation or private buffer exemption replaces any complete input.

Unknown/denied/unsupported/missing/corrupt/copied/released/uncertain originals stay
held. No body-selected store is adopted/opened or second policy database created.
Binary rollback does not restore World/Files original journals or live authority.

## Exact World SDK/native and Files delivery request

World owner must consume ACTUALLY published IPC0.1.9/world-policy-projection and mount
this typed schema on its existing selected reverse verifier, with explicit trusted
endpoint/token/timeout and new whole reply bounds. Source identity/store and complete
snapshot from actual SAME original material pin/Files authority, preserving complete
original Access independently of fresh observer. Use issued Policydev.4 decoder;
no private World/Files codec or body permissions. Implement actual current Auth and
same-store checks before/after awaits and owned+queued terminal journal verification.
Publish a normal owner-checked real SDK/native successor; its coordinate is selected
by that owner, never guessed here. Existing issued SDK0.2.1-dev.5 supports .7 opaque
witnesses only, so a new profile consumer is genuinely required.

Files then consumes that actual SDK successor with mandatory ipc-file-capability,
ipc-content, ipc-content-verifier/test-artifacts support preserved, plus real new
projection support. Files independently decodes with Policy0.1.0-dev.4 and mounts
its genuine same-authority bridge; retained startup/native acceptance is its own
unfinished owner delivery. Registry graph remains coherent DataRC3/SharedRC2/WorldRC11,
Policydev.4 (test Redbdev.3), with no decorative SDK dependency in this contract.
Fixtures use isolated real Redb with explicitly unsigned Auth/material adapters.
No native/startup/installed/Vertical acceptance or business credit from this carrier.

IPC0.1.9 changes only the optional decoder dependency to issued Policydev.4.
It preserves the complete0.1.8 carrier/schema/profile/wire. Policydev.4 accepts
canonical bare lowercase SHA256 physical descriptors with unchanged prefixed
original proofs, without rewriting either field or any original Access/held row.
IPC0.1.8 remains immutable and exactly pinned to Policydev.2; mixing it with
Policydev.4 under required projection features is not a coherent resolver graph.
