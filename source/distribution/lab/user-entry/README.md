# Standalone Omega process witness

[component/main.omg](component/main.omg) implements the common platform
`Program` entry. It checks the installed environment, yields, descends into
[channel.omg](component/channel.omg) for message/custody checks and
[shared.omg](component/shared.omg) for page ownership,
[notification.omg](component/notification.omg) for wake rights, and
[timing.omg](component/timing.omg) for clock authority and parking, then returns
an exit status. [native/main.omg](native/main.omg) is the compiler's independent
component consumer; its executable is packaging input, never a host program to
run. The actual process enters through the platform's native entry adapter.

Run from the repository root:

```powershell
python tools/omega-user/run_test.py
```

The runner boots the same Omega ELF twice under the ordinary Rust kernel,
with and without a clock grant. Each process has two transport objects of quota, no
initial endpoints, one private page and one reader page of allowance. It checks:

- The environment snapshot and yield return.
- A full 64-byte message, including its last word, and unchanged send backing.
- A third channel allocation is refused while two consume the quota.
- Both endpoints close; a retired sender fails; another exchange can reuse the quota.
- A live shared view survives producer release, retains its funding until the last
  reader releases, and then permits a new allocation.
- Wrong-type receives leave the grant queued; reader writes and retired names are
  refused; last-page access succeeds, crossing and overflowing spans fail.
- Delegated notification names are distinct; wait rights cannot signal; signal
  delivery, wait revocation, stale names and repeated cleanup/reuse work.
- Without a clock grant, both acquisition and parking are denied.
- With the grant, a two-tick deadline parks and resumes at or after the deadline.
- The kernel reports normal exit with status zero.

Artifacts and serial logs live in `build/omega-user/tests/<timestamp>/`.
The guest's proof is actual process execution, not a mocked table callback.

This witness establishes the entry/syscall foundation needed by standalone
Background. It does not test the matrix broker, shared mailboxes, notification
races between separate processes, image attachments or owner-control service.
Those remain required before changing the ordinary F5 Background executable.
