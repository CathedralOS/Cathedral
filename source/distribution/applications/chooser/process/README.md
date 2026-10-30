# Chooser process

Start at [main.omg](main.omg): install service/control connections, run
[application/application.omg](application/application.omg), then clean up.
[application/operations.omg](application/operations.omg) dispatches the app's
named commands to services, storage and presentation. The app itself remains
in `../app/`; it does not handle kernel objects or process bootstrap.

- [services/exchange.omg](services/exchange.omg) publishes once and services
  independent owner control while awaiting the selected peer.
- [services/requests.omg](services/requests.omg) scopes storage requests and
  retains outgoing attachments; [replies.omg](services/replies.omg) accepts only
  plain replies or bounded directory pages.
- [storage/entries.omg](storage/entries.omg) uses the platform's canonical
  metadata and ordered-row validator, then releases the incoming reader.
- [memory/payload.omg](memory/payload.omg) copies one bounded chunk;
  [presentation/images.omg](presentation/images.omg) sequences private
  allocation, contiguous writes, sealing and transfer.
- [services/waiting.omg](services/waiting.omg) preserves the refresh deadline
  while handling owner traffic. A failed owner reply retires that connection
  without wedging independent service calls.

The native package requests interaction and storage imports; its manifest
requests authority and never grants it. The platform user runtime owns syscall,
connection, deadline and attachment primitives. The native adapter is x86-64.

`tools/omega-user/chooser_test.py` boots the actual process against a controlled
Rust provider. It checks a full 51-row page and continuation, malformed metadata,
row order and extent, owner progress during a held storage call, image contents,
periodic refresh, and page/channel/notification refunds after cancellation.
The desktop and settings tests exercise the same executable against real matrix
and storage services. Exact qualification and compiler provenance live in
[TASKS_RUST.md](../../../../../TASKS_RUST.md).
