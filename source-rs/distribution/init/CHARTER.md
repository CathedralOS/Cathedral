# Cathedral initial program

Owns userspace startup, navigation, appearance and restart policy for the built-in
distribution. Boot supplies indexed display and input launch/connection grants;
init starts the independent providers and keeps selection/toggle state across
either restart. Arrows select, Enter toggles, F1 restarts input and F2 display.
Main orchestrates the scene; service lifetime and scene behavior live separately.
Healthy tasks block on events or requests. No intentional faults or exhaustive
test workloads run here. Detected request failures get three attempts; failed
initialization ends startup and hung providers are not yet monitored.

This is a bounded first init, not a shell, trusted recovery path or general service
manager. Kernel grants admit up to three approved children independently. A fork
can replace this program and its profile, including using no platform services.
