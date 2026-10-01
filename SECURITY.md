# Security boundary

## Trust model

Ready mode trusts exactly one reviewed native worker binary identified by an
absolute canonical path and lowercase SHA-256. A digest authorizes code; it
does not make an arbitrary binary safe. Operators must pin only a worker built
from the reviewed `nuxt-v8-view-worker` source and record its provenance.

The runtime and worker remain independent repositories. There is no Cargo path
dependency, copied V8 implementation, Hatter dependency, or runtime reference
to `RUSTY_V8_ARCHIVE`.

## Enforced controls

- configuration and protocol structs reject unknown JSON fields;
- simulation and ready settings are mutually exclusive;
- ready mode requires `local-production`, `external_egress: deny`, and
  `external_actions: false`;
- worker paths must be canonical absolute regular non-symlink executables;
- the opened file identity and SHA-256 are verified before descriptor
  execution;
- script ID, script digest, expiry, resource limits, and request correlation
  are pinned;
- child environment is cleared and all three standard streams are piped;
- input, output, diagnostic output, and wall-clock execution are bounded;
- output is checked against the worker response contract and the capability's
  exact session-band payload schema;
- no worker failure detail crosses the loopback HTTP authorization boundary.

## Egress boundary

The reviewed worker exposes no V8 host callbacks and accepts only four bounded
scalar input values. The protocol contains no URL, path, credential, command,
or arbitrary script field. The runtime itself has no HTTP client dependency.

`external_egress: deny` is an invariant and capability statement, not a
general-purpose sandbox for an unreviewed native executable. Deployments that
treat the pinned native worker as hostile must add an outer dedicated UID,
read-only filesystem, and OS network/seccomp policy. The runtime fails closed
on non-Linux systems because safe descriptor execution is currently
implemented only with Linux `/proc/self/fd`.

## Local artifacts

`examples/local-ready.generated.json` contains a device path and executable
digest, so it is gitignored. The template contains no usable digest. V8 static
archives are build inputs for the independent worker only and must not be
stored in this repository or referenced by runtime configuration.
