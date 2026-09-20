# Actual Mapper default and view witnesses

```sh
python3 tools/ports/x86_64-mapper-conveniences/check.py --omega /path/to/omega
```

The Rust recording implementation invokes the actual pinned Mapper/Translate
trait defaults and checks delegated arguments, error propagation and numeric
results. The generated route reference reuses the reviewed owned-table setup
from the mapping-route harness, changing only which public Mapper entry is
called and the fixed assertions. No translation root is installed.

`main.omg` covers value wrappers; `routes.omg` covers three mapping sizes;
`translate_route.omg` covers read-only translation and identity rejection.
`--controls-only` runs the same reference audits and four body mutations when
these positive fixtures have already passed. Every mutation changes an expected
algorithm result and must fail under the unchanged success contract.

See `source/libraries/x86_64/mapper-conveniences.PORT.md` for exact pin, source
mapping, counts and authority boundaries. Route/core/PTE implementations remain
owned by their existing modules.
