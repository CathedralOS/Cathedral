# Experimental display provider

Owns bounded software drawing into one boot-granted linear GOP framebuffer.
The separately compiled userspace service exposes dimensions, clear and rectangle
requests over existing IPC. It accepts no physical addresses or mapping requests.
Distribution clients choose colors, layout and restart policy. This is a device
bring-up protocol, not a compositor, trusted prompt path, capture API or stable
surface contract. Boot can inject a fault after a fixed request count for tests;
the client protocol has no crash command. Fault injection and negative probes
are compiled only with the service's `lab` feature, selected by the smoke profile.
