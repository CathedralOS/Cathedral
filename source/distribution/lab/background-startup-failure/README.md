# Background startup failure

[component/main.omg](component/main.omg) runs the real Background startup, prints
the failure marker only after opening owner control and broker transport, then returns a nonzero
exit status. It deliberately leaves acquired resources for process teardown.
The owning matrix still decides retry and fallback.

[native/main.omg](native/main.omg) packages the fixture with the same funded
imports and native adapter as Background. It is never installed by normal F5.
The [nested failure test](../../../../tools/boot-harness-rs/nested_failures_test.py)
substitutes it into a disposable disk when the selected profile uses native
Omega Background. That test checks bounded retries, usable inner chrome and an
unaffected outer Notes draft across repeated subtree teardown.

The native fixture passes both retry cycles in the nested failure test, alongside
the picker and user-matrix failures. Evidence and the current default composition
are recorded in [the active work](../../../../TASKS_RUST.md).
