# Parent interaction and retained image custody

[upstream.rs](upstream.rs) exchanges a packet and optional sealed image through an
explicitly supplied pair. The caller owns stream serialization and must retire it
on an ambiguous transport failure. This is not a service locator or a display policy.

[images.rs](images.rs) holds one reader per upstream surface. Validate room before
sending an image; commit its custody only after acceptance. Rectangles and erasure
release the replaced reader. Replay borrows the reader without mapping pixels.
The stock matrix's asynchronous presentation path commits through the same arena.

Consumers: [stock parent](../../../../../distribution/matrix/service/parent/parent.rs)
and [Split](../../../../../distribution/applications/split/session/rendering.rs).
Parent grant/input interpretation lives in the pure matrix library's
[upstream snapshot handler](../../../matrix/world/upstream.rs). Root-compositor
adaptation and transactional presentation orchestration still live in the stock
implementation; this extraction does not make those paths reusable yet.
