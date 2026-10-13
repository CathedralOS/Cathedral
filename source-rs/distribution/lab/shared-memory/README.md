# Ordinary shared-object forwarding

[main.rs](main.rs) dispatches three independent processes: [producer](producer.rs)
allocates/freezes a page; [forwarder](forwarder.rs) copies its read right without
mapping; [reader](reader.rs) maps after both predecessors exit. The kernel
[boot suite](../../../kernel/boot/uefi/lab/shared_memory.rs) verifies completion
order, one physical allocation, zero forwarder mappings, write-fault containment
and full reclamation. Discard and invalid-copyout probes exercise queue custody.

Run either full CPU proof from the repository root:

```text
python tools/boot-harness-rs/run.py --smoke --release --timeout 120
python tools/boot-harness-rs/arm64/run.py --release --timeout 120
```

Omit `--release` for debug. [The memory contract](../../../kernel/core/user_tasks/memory/README.md)
connects these probes to syscall validation, accounting and runtime wrappers.
