# Cathedral initial program

Owns userspace startup, navigation, appearance and restart policy for the built-in
distribution. Boot supplies indexed display and input launch/connection grants;
init starts the independent providers and keeps selection/toggle state across
either restart. Arrows select, Enter toggles, F1 restarts input and F2 display.
Main orchestrates the scene; service lifetime and scene behavior live separately.
Healthy tasks block between requests. Input returns explicit idle responses;
init uses those to check display health. Initialization, input replies and redraws
have deadlines; detected failures reclaim/restart only the affected provider.
Startup and requests each have bounded retries; exhausted retries end init and
reclaim its children. Scene state stays in init throughout provider recovery.
Normal builds have no deliberate faults. The recovery-lab feature permits fault
requests and verifies sibling task identity for the QEMU recovery harness.

This is a bounded first init, not a shell, trusted recovery path or general service
manager. Kernel grants admit up to three approved children independently. A fork
can replace this program and its profile, including using no platform services.
