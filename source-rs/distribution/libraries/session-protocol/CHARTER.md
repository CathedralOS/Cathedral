# Distribution session status

Owns the small init/status-app readiness and health protocol. Reports contain
observed provider/app generations and last recovery; they confer no control
rights. This is distribution composition, not a kernel contract or platform
service discovery API. Both consumers stay inside the replaceable distribution.

Named launch and app-link indices describe the fixed stock profile. See the
[composition map](../../README.md) for both endpoints and their protocols;
changing profile order requires updating these names together.
