# Cathedral initial program

Owns userspace startup and the initial scene for the built-in distribution. Boot
starts this one executable with a profile-selected launch grant; this program
chooses when to launch, connect, draw and restart. The platform display provider
is an independent executable. A healthy session remains alive with init waiting
for its child and the provider waiting for requests. No startup test suite or
intentional faults run here. Three failed service generations end the session.

This is a bounded first init, not a shell or general service manager. The current
kernel grant admits one approved child at a time. A fork can replace this program
and its profile, including selecting an initial program with no child services.
