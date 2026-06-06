# Phase 5: Components and Services

> Cathedral's "init": how the OS's own processes start, get their authority, and supervise each other, with no ambient power and no install scripts. Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented.

With the store mounted ([phase 4](04_mounting_the_store.md)), component code is reachable in the system realm's package store, and the kernel can start the userspace side of the OS.

## The root supervisor

The kernel starts one first component: the **root supervisor**, the top of a supervision tree ([component model](../design/part_2_components/00_component_model.md), [error model](../design/part_2_components/04_error_model_and_recovery.md)). This is Cathedral's equivalent of init, but it is a restartable component, not a privileged monolith. The thing that fails and the thing that restarts it are different components on purpose (the Erlang lesson).

## A declarative startup, not scripts

The supervisor reads a **typed, declarative startup configuration** from the system realm ([configuration & policy](../design/part_4_storage/02_configuration_and_policy.md)): which components to start, the capabilities each requires, their resource budgets, and their dependencies. There are no startup shell scripts; bringing the system up is a declarative state transition, the same principle as install ([package system](../design/part_5_lifecycle/00_package_system.md)).

## Spawning with explicit authority

Each component is spawned with an explicit initial state and an explicitly granted capability set ([instance creation](../design/part_2_components/00_component_model.md)). The supervisor delegates exactly the capabilities the component's manifest declares and nothing ambient, so a freshly started service can reach only what it was handed. Components are brought up in dependency order; a component that needs the network waits for the network stack, and so on.

## Drivers and services

- **Drivers** are components like any other ([driver model](../design/part_5_lifecycle/02_driver_model.md)): device discovery enumerates hardware, matching binds each device to a driver, and the driver is spawned with capabilities over *its* registers, DMA regions, and interrupt lines, nothing more.
- **Services** register a protocol and identity so others can find and invoke them ([IPC](../design/part_3_communication/00_ipc_and_service_invocation.md), [naming](../design/part_6_human_surface/03_naming_and_discovery.md)). Crash recovery is contained and the supervisor restarts the failed component.

## Where this phase ends

The system is now up: memory, scheduling, IPC, drivers, and core services are running, the authority graph is populated, and the machine is functional. But no human is present yet. What is running is the system realm and the components that brought it to life, sitting at the login surface.

## Next

[Phase 6: Session and login](06_session_and_login.md) — the human, their credentials, and unsealing the user realm.
