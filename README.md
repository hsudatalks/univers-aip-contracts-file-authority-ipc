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
