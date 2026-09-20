# UEFI network representations (UEFI-006)

## Scope and status

**Whole-slice stage: inventoried, with seven explicitly blocked operations or
fields.** The fixed raw transcription is typechecked. Pure helper tests execute
in Omega's semantic evaluator. Native execution, firmware interaction and emitted
Omega foreign-layout agreement are not established. Reviewed 2026-09-20 as
uncommitted Cathedral work following `950299b8e20cb0c921aea464cf7cf231218b4a4c`.

Claimed upstream slice: `uefi-raw/src/net.rs` and every file under
`uefi-raw/src/protocol/network/`: SNP, PXE, DHCPv4, IPv4 records/configuration,
TCPv4, HTTP and TLS configuration. This is the pin's surface: its `ip4.rs` leaves
an IPv4 protocol table as TODO, and its `tls.rs` contains configuration rather
than a TLS handshake table. No missing upstream protocol is invented. This is
not a Cathedral network stack.

- `network.omg`: 81 inert records, 165 pinned constants, explicit zero IP value,
  and one separately identified RFC2131 broadcast-bit fact.
- `network_layouts.omg`: 80 fixed foreign policy plans whose source checks pass.
- `network_large_layouts.omg`: full 34-field PXE mode plan, excluded from passing
  import roots because the current layout schema supports at most 32 fields.
- `../../../libraries/uefi/network_helpers.omg`: pure initialized-data behavior.
- Repository-root `tools/ports/uefi-network/`: isolated developer fixtures.

No production boot root imports these modules. Remaining integration includes
actual foreign-layout inspection, bounded runtime tails, checked borrowed packet
views, protocol invocation and authority/lifetime adapters. Those are not
implied by successful semantic tests.

## Upstream pin and licensing

The derivative source and translated test cases preserve `MIT OR Apache-2.0`
from [uefi-rs](https://github.com/rust-osdev/uefi-rs) commit
`c0facddf9ba42b74906a37fca2869e6cdbc8da6a` (`uefi-raw`, pinned workspace license).
Source headers identify modified translations. Exact license texts, crate
copies and notices are retained under
[licenses/rust-osdev/uefi-rs](../../../../licenses/rust-osdev/uefi-rs/) and indexed
in [THIRD_PARTY_NOTICES.md](../../../../THIRD_PARTY_NOTICES.md). Optional checkout:
`reference_code/rust-osdev/uefi-rs`; absence or pin mismatch fails the probes.
Primary ABI facts and RFC2131's bit definition remain distinct from the
upstream-derived API, algorithms, organization and test cases.

## Source and public-symbol map

[network-inventory.json](network-inventory.json) covers 10 files and 720 lexical
review anchors: **694 translated, 19 omitted, seven blocked, zero pending**.
Every field, associated constant, operation and test anchor is mapped. This
lexical source audit is not a Rust semantic parser. The companion
`tools/ports/uefi-network/schema.json` preserves every original record's field
order/type, union alternatives, packing, and its Omega carrier correspondence.

| Source group | Translation | Disposition |
| --- | --- | --- |
| `net.rs` | IP/MAC raw storage, conversions, zeroing and adapted tests | Fixed carriers and pure behavior translated; Rust display formatting omitted |
| `snp.rs` | Protocol slots, mode, flags, 26 statistics and optional getters | Translated as inert data and pure sentinel interpretation |
| `pxe.rs` | Full fixed records, packet storage, constants, endian/accessor/filter helpers | One runtime tail and two borrowed packet projections blocked; 34-field plan retained separately |
| `dhcp4.rs` | Protocol, state/event, packet/header/config/token records | Two flexible tails blocked; fixed prefix and tail offsets retained |
| `ip4.rs`, `ip4_config2.rs` | Pin's IPv4 configuration/mode/route and configuration protocol | Translated without adding the upstream TODO table |
| `tcp4.rs` | Protocol, tokens, connection state, configuration, packet carriers | Two fragment-array tails blocked |
| `http.rs`, `tls.rs` | HTTP representations/defaults and TLS configuration table | Translated; Debug presentation omitted |
| `mod.rs` | Flat `network` Omega module | Rust module namespace omitted with reason |

Existing Cathedral `Efi*` boot declarations retain their names and semantics;
these representations live only in the separate raw package/module.

## Primary specifications and representation vectors

Governing sources reviewed:

- [UEFI 2.11 §24, SNP and PXE](https://uefi.org/specs/UEFI/2.11/24_Network_Protocols_SNP_PXE_BIS.html).
- [UEFI 2.11 §28, TCP/IP and configuration](https://uefi.org/specs/UEFI/2.11/28_Network_Protocols_TCP_IP_and_Configuration.html).
- [UEFI 2.11 §29, DHCP and HTTP](https://uefi.org/specs/UEFI/2.11/29_Network_Protocols_ARP_and_DHCP.html).
- [RFC2131 §2, Figure 2](https://www.rfc-editor.org/rfc/rfc2131.html#section-2),
  BOOTP flags and network byte order.

Direct UEFI retrieval sometimes returned HTTP403; indexed official text was
available. Full primary verification of every upstream constant remains
incomplete. The exhaustive values are explicitly pinned-upstream observations,
not a claim that every value has been independently certified against UEFI.

[network.vectors.json](network.vectors.json), `cathedral-port-vectors-v1`, holds
**703 facts**: sizes, alignments, field offsets (including all union alternatives
and excluded tail positions), flags/discriminants, and GUID bytes. Target profile:
UEFI x86-64, 64-bit pointers, little-endian. Host-compiled Rust measures the
pinned crate; every number and GUID byte is independently asserted by compiled
Rust under `x86_64-unknown-uefi`. `measure.py` records rustc identity. The source
checker also computes C-profile geometry and compares authored plans; it does
not measure Omega's native or foreign layout. No other target is claimed.

A concrete disagreement is preserved: pinned
`PxeBaseCodeDhcpV4Flags::BROADCAST` is numeric `1`, and its helper byte-swaps then
masks that value. RFC2131's leftmost flag bit corresponds to numeric `0x8000`
after network-to-host conversion. `pxe_bootp_flags_upstream` preserves the pin;
`DHCP_RFC2131_BROADCAST_BIT` and `pxe_bootp_broadcast` express the primary fact
separately. A raw x64 flags field `0x0080` tests as primary broadcast true and
pinned flag result zero. This additional primary fact is not one of the 703
upstream measurements.

## Translated tests and fixtures

`tools/ports/uefi-network/main.omg` translates every pure `net.rs` test's relevant
cases: IPv4/IPv6 round trips, promised conversions, tagged input/output, MAC
32-byte and six-byte forms, full zero padding, all-16-byte IPv4 initialization,
and the mock receive value 42. The upstream unsafe mocked-pointer flow becomes
an owned initialized value flow; no foreign write, aliasing or call is tested.
Local cases cover all 26 statistics at zero/unavailable boundaries, high-bit
values, endian examples, 24-bit transaction IDs, PXE optional addresses, filter
counts 0/1/8, malformed count/index rejection, packet byte views, HTTP scalar
defaults and the primary/pinned broadcast discrepancy.

`check.py` runs `--check` on a constant initializer calling the actual Omega
helpers/test bodies and requires its evaluated result to equal zero. A separate
negative-control project changes that assertion to computed result plus one;
it must fail with `cannot prove requires contract` and `0 + 1 == 0`. This is
**semantic-evaluator execution**, not native execution. HTTP address defaults
are source/typechecked initialized carriers; no external pointer usability is
tested. Borrowed typed DHCP packet views remain blocked, not passed tests.

## Omega blockers and boundary seams

Reviewed Omega checkout: `eaa7993a23623cd8fabf45350340479c5c9c7879`.
Existing compiler artifact SHA-256:
`a87e533bc7c068419ab2ff05c213abbb4a874e3fe8dbba35e5636004e6f5bfe6`.
The artifact was not rebuilt and is not attributed to the checkout revision.

| Blocker | Affected surface | Evidence / exact reproducer |
| --- | --- | --- |
| `omega:runtime-layout-strides` | `Dhcp4Packet.option`, `Dhcp4PacketOption.data`, `PxeBaseCodeDiscoverInfo.srv_list`, TCP4 receive/transmit `fragment_table` | `Omega/wiki/spec/layouts/plans.md` leaves exact programmable runtime-stride source forms unspecified. All five original tail declarations and measured offsets remain in source/schema; only fixed prefixes are usable. No unchecked extent or reference is invented. |
| `omega:layout-reflection-capacity` | Complete `PxeBaseCodeMode` foreign plan | `omega --check tools/ports/uefi-network/layout_capacity.omg`: `schema data PxeBaseCodeMode has 34 fields; the current layout slice supports at most 32`. Importing the full plan alone also fails indices32/33 bounds. |
| `omega:plan-laid-public-fields` | Cross-package field projection and borrowed typed PXE packet views | `layout_projection.omg`: `selects private data NetworkStatisticsX64Layout<NetworkStatistics>::rx_total_frames`; `layout_union_view.omg`: `selects private data PxeBaseCodePacketX64Layout<PxeBaseCodePacket>::raw`. The latter attempts an ordinary checked shared-byte recast; further recast validation is unobserved because this earlier error stops it. No general union/recast design gap is inferred. |
| Foreign address materialization and invocation seam | All callback/service slots and pointer-bearing records | Raw `addr` values are inert. Omega authority/device-access/calling documentation requires explicit custody, storage lifetime, external mutability and call policy; this slice supplies no binding or service authority. Native/firmware execution is not attempted. |

The upstream unsafe union reads assume initialized bytes and the correct IP
family, while callback pointers assume firmware ABI, valid buffers, capacities,
aliasing rules and completion-event lifetimes. Pure helpers operate on initialized
owned bytes and explicit family selection. Async network buffers/tokens, firmware
mutation and callback custody require a future named boundary; an address or GUID
is never a capability. Shared packet references must retain source lifetime and
alignment through a checked view; blocked typed projections are not replaced by
unsafe casts.

## Deliberate deviations

- Rust associated constants become prefixed constants; open numeric enums and
  bitflags become nominal `raw` carriers, preserving unknown values.
- Rust pointers/function pointers become inert `addr`; full pinned signatures
  remain adjacent. `usize` is explicitly `u64` in this target profile.
- Untagged unions store full raw bytes/word/address; every alternative is listed.
  Fixed foreign plans provide required size/alignment separately. Default Omega
  layout is never assumed to be the C representation.
- Rust core IP/Option types become octet arrays and nominal `IpOctets`,
  `OptionalIpAddress`, `NetworkStatistic`. PXE optional-address getters return
  owned snapshots. These preserve pure values, not Rust trait/borrow identity.
- PXE filters accept capacity-eight data plus a bounded count; unused entries
  are zeroed. The accessor returns optional indexed values and rejects invalid
  raw counts/indices, replacing the upstream borrowed slice/panic behavior.
- Formatting/Debug and module glue are omitted. No network policy, socket stack,
  boot selection, TLS trust store or certificate interpretation is added.
- Five zero-length Rust flexible arrays are explicitly blocked, not silently
  approximated by fixed arrays. Their prefix/tail offsets remain auditable.

## Cathedral integration and authority

Owning charters: `source/contracts/CHARTER.md` (inert foreign representation) and
`source/libraries/CHARTER.md` (pure reusable behavior). Build roots are only
`source/contracts/uefi/raw/build.omg`, `source/libraries/uefi/build.omg`, and
`tools/ports/uefi-network/build.omg` when explicitly selected. The large plan is
outside passing fixture import graphs. No existing boot adapter is replaced.
Production canaries, hardware, simulator and native execution: not run for this
isolated corpus; no integration or service authority is claimed.

## Verification commands and results

Run from Cathedral root with the exact pin/artifact above:

| Command | Result and scope |
| --- | --- |
| `python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/uefi-rs source/contracts/uefi/raw/network-inventory.json` | PASS:10 files,694 translated,19 omitted,7 blocked,0 pending. `--require-transcribed` intentionally fails on the seven blockers. |
| `python3 tools/ports/uefi-network/measure.py` | PASS:703 upstream values/geometry/GUID bytes verified on Rust UEFI x64; no Omega ABI claim. |
| `python3 tools/ports/uefi-network/check_transcription.py` | PASS:81 raw records,165 pinned constants,81 authored plans match schema and independently verified vectors. |
| `python3 tools/ports/uefi-network/check.py --omega ../Omega/target/release/omega` | PASS:actual pure Omega test bodies evaluated; computed-result negative control rejected. |
| `../Omega/target/release/omega --check tools/ports/uefi-network/layouts.omg` | PASS:80 fixed policy definitions and their raw dependencies; no emitted foreign-layout observation. |
| `../Omega/target/release/omega --check tools/ports/uefi-network/layout_capacity.omg` | Expected blocked result:34 fields exceed32. |
| `../Omega/target/release/omega --check tools/ports/uefi-network/layout_projection.omg` | Expected blocked result:public scalar field becomes private on synthesized policy value. |
| `../Omega/target/release/omega --check tools/ports/uefi-network/layout_union_view.omg` | Expected blocked result:public packet raw storage becomes private on synthesized policy value. |

Reproduction/generator details are in
[tools/ports/uefi-network/README.md](../../../../tools/ports/uefi-network/README.md).
