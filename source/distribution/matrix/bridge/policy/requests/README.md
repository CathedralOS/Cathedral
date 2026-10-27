# Native request-policy codec

Start at [main.omg](main.omg): decode accepted-connection facts and the embedded
Matrix IPC words, call [request policy](../../../service/requests/main.omg), then
encode the operation or successful-owner effects. [control.omg](control.omg)
adapts owner child-control facts and decisions through its separate typed policy.

These codecs own the temporary native envelope, operation numbers and effect
bitfields. Real protocol parsing lives with the service's `wire.omg` decoders.
The unchanged Rust adapters retain authenticated requests, capabilities, resource
custody and pending replies; no handle is transferred through these values.
