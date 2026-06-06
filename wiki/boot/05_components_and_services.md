# Phase 5: Components and Services

> Cathedral's "init": how the operating system's own processes start, where their authority comes from, and how they supervise each other, with no ambient power and no startup scripts. Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented.

With the store mounted ([phase 4](04_mounting_the_store.md)), component code in the system realm is reachable, and the kernel can start the userspace side of the operating system.

## The root supervisor

The kernel starts one component first: the **root supervisor**. It is the top of a *supervision tree*, a structure where supervisor components watch worker components and restart them when they fail ([component model](../design/part_2_components/00_component_model.md), [error model](../design/part_2_components/04_error_model_and_recovery.md)). This is the role init or systemd plays on a Unix system, except it is an ordinary restartable component, and the thing that fails is deliberately separate from the thing that restarts it.

## What "fail" means here

A reader might reasonably ask what is left to go wrong, given that these components are written in a proof-carrying language. Expected, typed errors (an absent file, a refused connection, a declined allocation) are handled in band by the code that meets them and never reach a supervisor. A *fault* is the case the supervisor exists for: a component that can no longer be trusted to make correct progress. Proof removes large classes of fault (memory corruption, capability escapes, many logic bugs), but not the rest: hardware errors and dying devices, resource exhaustion, a wrong assumption at a firmware or hardware boundary, a deadlock, or an invariant a bit flip violated after the proof assumed correct hardware. Foreign application code in another language can simply crash. For a component whose own state is no longer trustworthy, the recovery is a restart to a known-good state ([error model](../design/part_2_components/04_error_model_and_recovery.md)).

## A declarative startup, not scripts

The supervisor reads a typed, declarative startup description from the system realm ([configuration & policy](../design/part_4_storage/02_configuration_and_policy.md)): which components to start, the capabilities each needs, its resource budget, and what it depends on. There are no startup shell scripts. Bringing the system up is a declarative state transition, the same principle as installing a package ([package system](../design/part_5_lifecycle/00_package_system.md)).

## Spawning with explicit authority

Each component is created with an explicit initial state and an explicitly granted set of capabilities ([component model](../design/part_2_components/00_component_model.md)). The supervisor hands a component exactly the capabilities its manifest declares and nothing ambient, so a freshly started service can reach only what it was given. Components come up in dependency order: one that needs the network waits for the network stack to be ready.

## Drivers and services

- **Drivers** are components like any other ([driver model](../design/part_5_lifecycle/02_driver_model.md)). Device discovery enumerates the hardware, matching binds each device to a driver, and the driver starts with capabilities over its own registers, its DMA (direct memory access) regions, and its interrupt lines, and nothing else.
- **Services** register a protocol and an identity so other components can find and call them ([IPC](../design/part_3_communication/00_ipc_and_service_invocation.md), [naming](../design/part_6_human_surface/03_naming_and_discovery.md)). When one crashes, the failure is contained and the supervisor restarts it.

## Where this phase ends

The system is now running: memory, scheduling, messaging, drivers, and core services are up, the authority graph is populated, and the machine works. But no human is present. What is running is the system realm and the components that brought it up, sitting at the login surface.

## Next

[Phase 6: Session and login](06_session_and_login.md).
