# Standalone Omega process witness

[component/main.omg](component/main.omg) implements the common platform
`Program` entry. It checks the installed environment, yields, descends into
[channel.omg](component/channel.omg) for message/custody checks and
[timing.omg](component/timing.omg) for clock authority and parking, then returns
an exit status. [native/main.omg](native/main.omg) is the compiler's independent
component consumer; its executable is packaging input, never a host program to
run. The actual process enters through the platform's native entry adapter.

Run from the repository root:

```powershell
python tools/omega-user/run_test.py
```

The runner boots the same Omega ELF twice under the ordinary Rust kernel,
with and without a clock grant. Each process has one channel of quota, no
initial endpoints and no private/shared allocation grant. It checks:

- The environment snapshot and yield return.
- A full 64-byte message, including its last word, and unchanged send backing.
- A second channel allocation is refused while the first consumes the quota.
- Both endpoints close; a retired sender fails; another exchange can reuse the quota.
- Without a clock grant, both acquisition and parking are denied.
- With the grant, a two-tick deadline parks and resumes at or after the deadline.
- The kernel reports normal exit with status zero.

Artifacts and serial logs live in `build/omega-user/tests/<timestamp>/`.
The guest's proof is actual process execution, not a mocked table callback.

This witness establishes the entry/syscall foundation needed by standalone
Background. It does not test the matrix broker, shared mailboxes, typed grant
transfer, notification races, image attachments or owner-control service.
Those remain required before changing the ordinary F5 Background executable.
