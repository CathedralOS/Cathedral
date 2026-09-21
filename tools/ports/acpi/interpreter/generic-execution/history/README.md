# Historical generic-execution inputs

`scratch-success.tar.gz` retains the complete successful scratch bundle with its
original receipts, exact source/tool inputs and SHA manifest. These are scratch
results, not canonical-path replay. Its original execution root is recorded in
the archived handoff; no receipt has been rewritten to claim canonical execution.

`pre-generic-canonical.tar.gz` retains the prepublication canonical files plus
four original execution/pipeline receipts and their exact recorded source inputs
under `recorded-inputs/`. Inputs differing from the prepublication tree were
recovered from Git by matching each recorded SHA-256. All46/33/51/45 source inputs
were recovered. These receipts remain historical; they are not current generic
runtime proof.

Current replay receipts will be written outside this history directory. No native
or hardware execution is claimed by either archive.
