# Login handoff observation fixture

[process/main.omg](process/main.omg) selects the ordinary stock matrix with a
100-tick observation hold after the picker has been collected and before the
selected desktop's binding and placement are installed. It does not delay kernel
collection. The production entry selects zero; process arguments cannot enable
this fixture.

[omega_handoff_test.py](../../../../tools/boot-harness-rs/omega_handoff_test.py)
uses the Rust parent's `handoff-probe` feature to route HEALTH, STOP and policy
revocation/restoration through an outer Omega desktop to the nested login. The
new desktop must remain unserved throughout the hold. Cancellation must restore
the picker; a later selection must run Notes, save, reopen and survive cold boot.

Run the test with `--mode health`, `--mode stop` and `--mode regrant` after building
this [native entry](native/main.omg). The fixture uses the stock stack, grants and
deadlines. Its hold stays below the ordinary child's handshake deadline.
