# Original Artifact physical receipt query

The optional `artifact-physical-effects` feature exposes
`content::physical_effects` using Registry Data `=2.0.0-rc.6` through the
independent `univers-aip-contracts-data-effects` alias. The existing Data 1.x
features and dependency ranges remain intact. This crate supplies pure wire
values, not a Content provider, authentication, persistence or attempt closure.

The independent endpoints are `/v1/content/artifact-physical-effects/query` and
`/v1/content/artifact-physical-effects/capabilities`. The transport schema is
`univers.content-artifact-physical-effects/v1`; the negotiated capability is
Data's `artifact-physical-effects-v1`. Capability advertisements distinguish
`unsupported` from `query`. A provider may advertise `query` only after genuine
original receipt lookup is configured and implemented. Compiled features and
client-supplied capability names do not establish support or permission.

Requests use the exact Data query JSON. Replies echo that complete selector and
carry either a Data physical observation or a typed error. Use `decode_request`,
`encode_request`, `Reply::from_json` and `Reply::to_json` at the boundary to enforce
validation, strict JSON fields, request/reply correlation and total encoded byte
limits. Requests and replies are each bounded to 64 KiB, including the reply
envelope; error reasons are at most 1,024 UTF-8 bytes without padding or controls.
Unsupported schema/version, changed original scope, attempt/write/intent or
Content receipt lineage and malformed/trailing/oversized JSON fail validation.

`not_recorded`, `pending`, `staged`, `published` and `ambiguous` remain distinct.
Published receipts preserve the original Created/Reused outcome and bytesWritten,
including lossless decimal-string u64 encoding and separate logical/physical
hashes and sizes. Later digest reuse cannot replace an original receipt.
NotRecorded and errors never prove durable rejection, no effects or completeness.
This participant lane exposes no aggregate coverage, page, seal, bound write,
recovery, repair or close operation and has no fallback to legacy Content/head,
File catalog, retention or World policy projection.

The actual provider must authenticate the current actor, invoking owner and scope,
check the selected World/incarnation and genuine Content receipt lineage, enforce
budgets and retrieve durable originals. FILES-SWA shared-service authorization and
physical isolation remain Files/Framework responsibilities; these correlation
values cannot grant access or substitute for verified trusted context. Legacy or
unconfigured providers return typed Unsupported/Unavailable, never an invented
lineage or empty complete result. Historical unbound writes remain unknown.

Wire tests use unsigned synthetic fixtures; shape validation is not producer
truth or installed runtime acceptance. Files consumers require a separately
published FileIPC successor and provider delivery before enabling this lane.
