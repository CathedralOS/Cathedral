# Experimental display provider

Owns bounded software drawing into one boot-granted linear GOP framebuffer.
The separately compiled userspace service exposes dimensions, clear, rectangle
and printable ASCII text requests over copied IPC. Text uses the platform's
original bitmap font; every packet and its entire geometry are checked before
any device write. It accepts no physical addresses or mapping requests.

A private control connection serves the designated supervisor; a boot-approved
client link serves the application. The provider multiplexes both with bounded
readiness waits and accepts a fresh data connection after app replacement.
Client loss leaves the provider and its control connection alive. Linked service
composition requires an explicit clock grant. The legacy single-peer smoke
fixture requires none.

Distribution clients choose colors, layout and restart policy. Whole-screen
drawing remains a lab capability, not a compositor, trusted prompt path, capture
API or stable surface contract. `lab` adds boot-controlled smoke faults and mapping
probes; `recovery-lab` adds faults only on the private control channel. Neither is
included in ordinary builds, and client messages cannot trigger those handlers.
