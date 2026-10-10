# Future Artifact causal query: File Authority IPC boundary

Date: 2026-10-05. Owner: `univers-aip-contracts-file-authority-ipc`, `develop`.
Status at 2026-10-05: source preparation only. No future causal endpoint, producer binding,
physical receipt persistence, recovery, attempt dispatch barrier or seal is
implemented or published by this assessment.

## 2026-10-10 WIP completion

The source candidate is integrated in FileIPC 0.1.16 as the optional
`artifact-physical-effects` feature and `content::physical_effects` module, with
wire tests and fixtures in `tests/`. Registry Data 2.0.0-rc.6 is issued and its
physical query values match the original candidate; the alias is Registry-only.
The 0.1.15 upstream dependency ranges and legacy encodings are preserved.
[CONTENT_PHYSICAL_EFFECTS.md](../CONTENT_PHYSICAL_EFFECTS.md) describes the current
contract. The dated assessment below records the original preparation state,
including its then-unissued dependency and deferred checks. This source delivery
adds no provider endpoint implementation, publication or installed acceptance.
FILES-SWA admission remains an independent provider prerequisite.

## Actual source and compatibility

Intake was clean at `b0812f2854035c399db9b207cccebc2dd032bbc5` (0.1.5). A normal
fetch and fast-forward preserved all ten upstream commits and moved the source to
`66fd5c7` (0.1.14). The existing graph fixes Data `=1.0.0-rc.3`; optional retention
and World verification use WorldIPC `=0.3.12-dev.12`, and optional policy projection
uses Files Policy `=0.1.0-dev.6`. None is a future Artifact causal producer contract.
A point-in-time process-cwd inventory found only this worker's inventory process;
that observation is not a lock and does not authorize stopping another process.
No owner-local AGENTS.md or CLAUDE.md was found. Workspace instructions and the
Development Charter apply; the independent repository's normal hooks are required.

Legacy `Request`/`Reply`, `content::{Request, Reply}`, capability, retention,
world-verifier and world-policy-projection must keep their existing types,
features, codecs, bounds and old golden fixtures. An Artifact causal opt-in may
not inject fields into old Content Put/Upload requests and then permit an old
receiver to discard those fields while claiming causal support.

World owns File catalog/coordinator and Artifact catalog authority. Files owns
Content byte publication and the genuine physical receipt store. This crate owns
transport shapes only. Its existing Content put receipt is an operation result,
not a durable original-attempt receipt. File pending-write reconciliation, current
file/head/digest existence and historical retention verifier witnesses cannot
be relabeled as original Artifact physical receipts or an Artifact attempt seal.

## Phase-one transport boundary

Inputs are Files `docs/FUTURE_ARTIFACT_CAUSAL_RECEIPTS_20261005.md`, World
`.univers/governed-evidence-20261005/future-artifact-causal-world-boundary-answer.md`,
and the Data and ArtifactIPC owner handoffs. The Data owner's new candidate is
still an unissued draft at this assessment; no future version is pinned and no
schema is copied or guessed here.

The first permitted implementation is an independent optional readonly query
lane using the exact delivered Data original-scope/query/page/error values and
protocol. It must advertise only the operations actually implemented by its
provider. An unavailable/unconfigured provider gives typed unsupported/unknown,
never an empty complete result. It cannot fall back to legacy head, catalog scan,
Content Put, retention verification or repair operations.

The query selector preserves original request **and** attempt, invoking owner,
organization, World, namespace and explicitly typed authority incarnation.
Authority incarnation, selected revision and canonical Relation journal generation
are not Artifact producer generation, Content receipt lineage or dispatch seal.
A World-incarnation service observation may supply a typed scope component only
through its genuine configured authority and authorized current transport.
Client correlation fields, deserialization or a signed-looking body grant no
permission. Current Auth, actual selected World and participant scope remain
provider obligations; an absent actor/custody/incarnation supplier is a concrete
World/Framework dependency, never a locally minted fallback.

Pages preserve every admitted write's full original intent and immutable physical
receipt (including original Created/Reused and original bytesWritten), distinct
catalog linkage, unresolved intent/rejection and coverage. Metadata and logical
and physical digest/size distinctions cannot be projected away. Replaying an
original receipt must not replace it with a later digest reuse result. Unknown
IPC outcomes do not become durable pre-dispatch rejection.

A complete representation is permitted only with positive exact durable closed
manifest, producer admission/dispatch fence, producer/receipt generations, all
terminal members and stable snapshot evidence. Schema validation checks shape
and correlation; it cannot authenticate receipts or execute that barrier.
Pagination exhaustion, empty catalog, timeout, cancellation, process death, TTL,
a current head or a Relation journal generation cannot establish completeness.
A complete empty set covers only its exact authenticated closed producer boundary.
Legacy records without original binding remain unknown, including the historical
Economic native attempts; there is no retrospective backfill.

This readonly phase supplies no bound-write, signature issuance, close, publication
recovery or participant mutation Port. Those require actual World pending-before-
dispatch registration and Content original durable intent/receipt recovery with a
frozen compatible public contract. In particular a participant physical query may
not invent Artifact-wide completeness from its own digest store. Data and World
must define the participant query/result boundary before such a transport is added.

## Delivery sequence and verification

1. Data freezes and delivers exact additive opt-in values, feature names, protocol,
   full-intent encoding and validation. Preserve the existing Data rc.3 graph;
   use an independent compatible dependency graph only when the owner supplies it.
2. ArtifactIPC and this owner align explicit negotiation, budgets, strict request/
   page/snapshot correlation, lossless integer encoding and typed unsupported.
   No private socket, guessed canonical hash or silent old-wire downgrade is added.
3. This owner adds only the supported readonly transport and meaningful wire
   fixtures. Preserve all old golden cases. Query cases must reject changed exact
   scope, malformed/oversized values, unsupported protocol, conflicting original
   identity and mixed snapshots. Test unknown/partial and positively justified
   complete representation separately; contract tests do not claim producer truth.
4. Obtain the coordinator's heavy slot before Cargo, owner checks, hooks or package
   verification. Run the normal `scripts/check.sh` and both Git hooks, fetch and
   integrate external changes before delivery, commit/push clean `develop` without
   bypassing hooks. A source candidate is not an issued Registry dependency.
5. Handoff exact version/SHA, supported features, dependency graph and checks.
   World/Files consumers wait for the published compatible combination. Genuine
   producer durability, signed provenance, recovery and installed acceptance are
   separate owner work. Development only; no native pointer or stable promotion.

At this assessment only metadata, source reading, fetch/fast-forward and process-cwd
inventory ran. No Cargo/build/test/hook/package/native/performance/runtime mutation
ran. This document awaits source dependencies and normal owner delivery gates.

## Concrete source candidate after Data handoff

Data supplied actual source-ready, unverified/uncommitted/unpushed/unissued
`2.0.0-rc.5`, baseline `adb3d2e20bb34804001077d0d681890cc4bc697c`, with
`artifact_effects/physical_query.rs` SHA256
`977849e0731f946f0cadf2262abe95bd55dfc14a85d105baf53aa0c314293343`.
The delivered model is a distinct exact-write physical query, not an aggregate
Artifact page. Its `ArtifactPhysicalEffectQuery` carries schema/version, exact
write binding, intent digest and expected authority generation. Observation echoes
that whole query and distinguishes NotRecorded/Pending/Staged/Published/Ambiguous.
Its request/result JSON limits are each 64 KiB; its query Port uses a separate
non-wire trusted context. No participant coverage/complete value exists.

This repository's prepared, uncompiled source is under `docs/source-candidates/`:

- `content_physical_effects.rs`: pure readonly transport using the actual Data
  values through a future independent Registry alias, no duplicated Data schema.
- `physical_effects_wire.rs`: pending test source with exact-scope/attempt/write/
  intent/lineage denial, explicit unsupported, unknown/staging/ambiguity, preserved
  original Created/max-u64/protected byte differences, strict bounds and JSON.
- `physical_query.json`, `physical_published.json`: unsigned source fixture values;
  no production authority or physical publication is claimed by their contents.

The proposed independent endpoints are
`/v1/content/artifact-physical-effects/query` and
`/v1/content/artifact-physical-effects/capabilities`, wire schema
`univers.content-artifact-physical-effects/v1`, using Data's distinct
`artifact-physical-effects-v1` capability. The exact query JSON is the request;
the result includes transport schema, the full sent query and an observation or
typed error. Total response including envelope is bounded to 64 KiB, error reason
to 1,024 bytes. Error never denotes a durable pre-dispatch rejection. Negotiation
can advertise only Unsupported/Query, never producer write/recovery/closure.
Owners must explicitly enable this lane only after actual provider implementation;
compilation or advertised names alone do not authenticate support.

No Cargo alias, feature or active module/test registration has been added. The
legacy crate stays unchanged and buildable against its existing issued graph.
After Data issuance, add its exact Registry-only epoch2 alias, optional
`artifact-physical-effects` feature and serde_json; integrate the candidate as
`content::physical_effects` and wire the tests into `scripts/check.sh`'s isolated
feature matrix. Bump this owner from current 0.1.14 to a fresh actual successor
only in that integration batch, generate the lockfile normally, and run allocated
normal owner checks/hooks/consumer/package verification before commit/push.
A source-ready draft cannot satisfy `--locked` or Registry consumer acceptance.
No private path/Git patch or invented package version is permitted as a shortcut.

`rustfmt --edition 2021` formatted the two prepared source files; Python parsed the
two JSON fixtures and recorded their SHA256 values. These are light syntax/format
preparation only. Neither test source nor Cargo was executed. No normal owner gate,
commit, push, package, publication or installed acceptance is complete yet.
