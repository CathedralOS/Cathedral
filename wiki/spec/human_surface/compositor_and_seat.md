# Compositor and seat

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-10-23 |

## Replaceable desktop programs

The owner accepted on 2026-10-23 that everyday chrome, backgrounds and other
desktop programs are replaceable. Background is a program role; it need not be
content rendered inside a monolithic chrome. The system realm supplies default
programs, and users can select custom programs within their granted authority.
Override slots are a possible configuration mechanism; their names, locations,
schema and resolution precedence are not prescribed by this contract.

The distribution owns the shipped default programs and presentation. Common
platform mechanisms enforce display/input authority and the protected recovery
route. This does not mandate a second platform-owned desktop or prevent a fork
from replacing the distribution's implementation.

## Protected outer recovery chrome

The shipped outer desktop retains an OS recovery chrome reachable by a long press
of the physical OS key. Selecting custom chrome hides the recovery chrome; it
does not remove it or give the replacement authority to disable its invocation.
Ordinary background or chrome overrides cannot replace the protected recovery
target. Recovery chrome need not render while hidden.

The gesture must be recognized outside the replaceable desktop's input path and
failure scope. A custom chrome, foreground app or nested matrix cannot consume
or suppress it. On invocation, the enclosing system presents the recovery chrome
and routes its interaction independently of the broken replacement. The required
ownership, execution resources and display/input access must remain available
when that replacement or its matrix hangs or crashes.

This is recovery from a failed replaceable environment, not a promise to recover
from failure of the kernel or every underlying input/display provider. Exact
press duration, recovery controls, restart policy and protected presentation
mechanisms remain implementation work. The contract requires availability, not
a particular process residency strategy.

## Nested matrices

An inner matrix need not install recovery chrome. It may receive an authorized
copy of a system realm containing defaults, and choose its own chrome, background
and other overrides within its authority. Supplying such a copy is optional;
there is no mandatory global system realm visible to all descendants.

Program availability does not grant control of physical input or the outer
recovery route. An inner recovery interface, if supplied, controls only its own
authorized environment. The outer OS-key escape remains owned by the enclosing
system regardless of inner configuration.

## Evidence and limits

The [Rust distribution](../../../source-rs/distribution/README.md) runs separate
chrome and background programs and recursively hosted desktops. Its current
launcher recovery and owner-mediated close are partial mechanism evidence; the
protected OS-key long-press recovery path described here is not implemented.

This acceptance does not settle general seat/pointer semantics, secure prompt
and observation-indicator protocols, capture exclusions, or a complete compositor
interface. These remain distinct from replaceability and the recovery requirement.
