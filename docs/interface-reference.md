# rust-v8-local-runtime interface reference

Use the [usage guide](getting-started.md) for the first steps. This reference preserves the current interface details and operational limits. Run command examples from the repository root, after preparing the exact declared dependencies and registered configuration.

## Payload contracts

Every capability owns one allowlisted payload schema:

```json
{
  "payload_schema": {
    "id": "nuxtjp.workflow-issue-summary.v1",
    "fields": [
      { "name": "open_count", "kind": "count" },
      { "name": "source", "kind": "text" },
      { "name": "local_only", "kind": "boolean" }
    ]
  }
}
```

Fixtures and engine results must carry the same `payload_schema_id`. Payloads
must be objects with exactly the declared keys. Values are limited to bounded
text, unsigned integer counts, and booleans; arrays, nested objects, null,
floating-point values, undeclared keys, and omitted keys are rejected. The
host validates fixtures at startup and validates engine output again before it
crosses the loopback-browser boundary.

Schema and field identifiers are limited to 128 ASCII alphanumeric,
underscore, hyphen, or dot characters. A schema contains 1–32 unique fields
and text values are limited to 4096 UTF-8 bytes. Reusing one schema ID with a
different field definition is rejected.

## Engine modes

`local-simulation` remains the portable default. It accepts only
`engine_status: simulated`, configured fixture views, and no process settings.
The NERP and Hibee examples and Canonical smoke stages continue to use this
mode.

`local-production` is an explicit opt-in. It accepts only
`engine_status: ready`, no simulation fixtures, and a
`nuxtjp://local-runtime/process-engine/v1` configuration. The host does not
link V8 or depend on worker source. It implements only the worker's versioned
stdio JSON protocol.

Before every invocation the host checks:

- an absolute canonical regular non-symlink executable path;
- the executable SHA-256 using the same file descriptor that will be run;
- the fixed script ID and script SHA-256;
- configuration expiry and bounded input, output, heap, and timeout limits.

On Linux the verified descriptor is executed through `/proc/self/fd` with an
empty environment, bounded stdin/stdout/stderr pipes, and a host-side kill
deadline. Unknown JSON fields, response identity changes, correlation changes,
and payload-schema changes fail closed.

## Closed boundary

- numeric loopback bind only;
- exact allowed Origin and audience;
- random in-memory session tokens;
- capability, classification ceiling, expiry, nonce, byte, and message checks;
- no HTTP client or other outbound-network dependency;
- no provider credentials;
- no remote persistence;
- no implicit connection to Hatter or any ecosystem product.

## Commands

`validate-config` does not start a server. It emits one
`nuxtjp://local-runtime/config-validation/v1` JSON result.

```bash
cargo run --locked --offline -- validate-config examples/local-simulation.json
cargo run --locked --offline -- simulate examples/local-simulation.json
cargo run --locked --offline -- serve examples/local-simulation.json
```

`serve` is a foreground process. The example binds to
`127.0.0.1:37843`; stop it with `Ctrl+C`.

### Opt-in ready validation

Copy `examples/local-ready.template.json` to
`examples/local-ready.generated.json`, then replace `worker_path` with the
absolute built worker path and `worker_sha256` with `sha256sum` output. The
generated file is ignored because it is device-specific.

```bash
cargo run --locked --offline -- \
  validate-config examples/local-ready.generated.json
cargo run --locked --offline -- \
  render examples/local-ready.generated.json
```

`render` processes one complete session/view flow without opening a server.
It is rejected for simulation configurations, just as `simulate` is rejected
for ready configurations.
