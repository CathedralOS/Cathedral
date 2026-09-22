#!/usr/bin/env python3
"""Pinned public logical-opcode observations using the unchanged service-trap driver."""

import argparse
import hashlib
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
import fixtures

ROOT = fixtures.ROOT
PIN = "257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5"
PROBE = ROOT / "tools/ports/acpi/aml-public-execution"
DEFAULT_TOOLCHAIN = Path("/Users/zcanann/.rustup/toolchains/nightly-2026-09-04-aarch64-apple-darwin/bin")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def snapshot():
    paths = [
        Path(__file__).resolve(), HERE.parent / "fixtures.py", HERE.parent / "execution-cases.json",
        PROBE / "src/main.rs", PROBE / "Cargo.toml", PROBE / "Cargo.lock",
        HERE.parent.parent / "mid-execution/fixtures.py", fixtures.NAMED / "fixtures.py",
        fixtures.GENERIC / "fixtures.py", fixtures.GENERIC / "decoder_fixtures.py",
        HERE.parent.parent / "execution/fixtures.py",
        HERE.parent.parent / "to-integer-execution/fixtures.py",
        HERE.parent.parent.parent / "pipeline/fixtures.py",
    ]
    toolchain = HERE.parent / "toolchain.json"
    if toolchain.exists():
        paths.append(toolchain)
    return {str(path.relative_to(ROOT)): sha(path) for path in paths}


def selection():
    rows = fixtures.execution_cases()
    assert rows == json.loads((HERE.parent / "execution-cases.json").read_text()), "Fixture JSON is stale."
    assert len({row["name"] for row in rows}) == len(rows)
    selected, omitted = [], {}
    for row in rows:
        assert row["kind"] == 1 and row["bytes"] == "" and row["bits"] in (32, 64)
        assert isinstance(row["number"], int) and 0 <= row["number"] <= (1 << row["bits"]) - 1
        assert isinstance(row["error"], str) and isinstance(row["after"], dict)
        assert isinstance(row["object_count"], int) and isinstance(row["setup"], list)
        assert bytes.fromhex(row["table"])
        if row["setup"]:
            omitted[row["name"]] = "Direct canonical store edit has no equivalent public API fixture."
        elif row.get("mutate_source"):
            omitted[row["name"]] = "Post-load source mutation has no equivalent in the unchanged public driver."
        elif row["name"] == "debug_target":
            omitted[row["name"]] = "Debug callback lies outside this service-free observation profile."
        else:
            selected.append(row)
    return selected, omitted


def expected(row):
    return {
        "result": "integer:" + str(row["number"]),
        **{"after:" + key: "integer:" + str(value) for key, value in row["after"].items()},
    }


def observations(stdout):
    pairs = [line.split("\t", 1) for line in stdout.splitlines()]
    assert all(len(pair) == 2 for pair in pairs)
    assert len({pair[0] for pair in pairs}) == len(pairs), "Duplicate public observation key."
    return dict(pairs)


def agrees(row, values):
    # Primary error categories are retained verbatim, with no guessed Rust mapping.
    return row["error"] == "Success" and values.get("load") == "ok" and all(
        values.get(key) == value for key, value in expected(row).items()
    )


def verify(record, require_binary):
    assert record["source_sha256"] == snapshot() and record["source_unchanged"]
    assert record["upstream_revision"] == PIN
    rows, omitted = selection()
    assert omitted == record["omitted"]
    assert list(record["rows"]) == sorted(row["name"] for row in rows)
    all_rows = fixtures.execution_cases()
    assert record["omitted_primary_fixtures"] == {row["name"]: row for row in all_rows if row["name"] in omitted}
    upstream = Path(record["upstream_source"])
    assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=upstream, text=True).strip() == PIN
    assert all(sha(upstream / path) == value for path, value in record["upstream_sha256"].items())
    for row in rows:
        observed = record["rows"][row["name"]]
        assert observed["primary_fixture"] == row
        assert observed["table_hex"] == row["table"] and observed["bits"] == row["bits"]
        assert observed["omega_outcome"] == row["error"] and observed["omega_expected"] == expected(row)
        values = observations(observed["stdout"])
        assert values == observed["observed"] and observed["exit_code"] == 0
        assert values["forbidden_calls"] == "0" and values["created_mutexes"] == "1"
        assert agrees(row, values) == observed["agrees_on_value_and_state"]
    assert record["agreements"] == sum(row["agrees_on_value_and_state"] for row in record["rows"].values())
    assert record["nonagreements"] == sorted(name for name, row in record["rows"].items() if not row["agrees_on_value_and_state"])
    if require_binary:
        assert sha(Path(record["binary"])) == record["binary_sha256"]
    print("PASS", len(rows), "public observations;", record["agreements"],
          "value/state agreements;", len(record["nonagreements"]), "retained nonagreements;",
          len(omitted), "explicit omissions")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--acpi-source", type=Path, default=ROOT / "reference_code/rust-osdev/acpi")
    parser.add_argument("--toolchain", type=Path, default=DEFAULT_TOOLCHAIN)
    parser.add_argument("--target-dir", type=Path, default=Path("/tmp/cathedral-logical-public-target"))
    parser.add_argument("--record", type=Path, default=HERE / "verification.json")
    parser.add_argument("--verify", action="store_true")
    parser.add_argument("--require-binary", action="store_true")
    args = parser.parse_args()
    if args.verify:
        return verify(json.loads(args.record.read_text()), args.require_binary)

    rows, omitted = selection()
    upstream = args.acpi_source.resolve()
    assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=upstream, text=True).strip() == PIN
    assert not subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=no"], cwd=upstream, text=True).strip()
    bound = snapshot()
    original = {str(path.relative_to(upstream)): sha(path) for path in sorted((upstream / "src").rglob("*.rs"))}
    original["Cargo.toml"] = sha(upstream / "Cargo.toml")
    manifest = (PROBE / "Cargo.toml").read_text().replace("../../../../reference_code/rust-osdev/acpi", str(upstream))
    cargo, rustc = args.toolchain / "cargo", args.toolchain / "rustc"
    toolenv = dict(os.environ, RUSTC=str(rustc))
    recorded = {}
    with tempfile.TemporaryDirectory(prefix="cathedral-logical-public-") as directory:
        work = Path(directory)
        (work / "src").mkdir()
        (work / "src/main.rs").write_bytes((PROBE / "src/main.rs").read_bytes())
        (work / "Cargo.toml").write_text(manifest)
        (work / "Cargo.lock").write_bytes((PROBE / "Cargo.lock").read_bytes())
        target = args.target_dir.resolve()
        command = [str(cargo), "build", "--offline", "--locked", "--release", "--jobs", "2",
                   "--manifest-path", str(work / "Cargo.toml"), "--target-dir", str(target)]
        built = subprocess.run(command, env=toolenv, cwd=upstream, capture_output=True, text=True)
        assert built.returncode == 0, built.stdout + built.stderr
        binary = target / "release" / ("cathedral-acpi-public-execution.exe" if os.name == "nt" else "cathedral-acpi-public-execution")
        binary_hash = sha(binary)
        for row in rows:
            aml = work / (row["name"] + ".aml")
            aml.write_bytes(bytes.fromhex(row["table"]))
            run = subprocess.run([str(binary), str(aml), "1" if row["bits"] == 32 else "2", "", *row["after"]],
                                 capture_output=True, text=True, timeout=5,
                                 env=dict(os.environ, CATHEDRAL_GENERIC_DESCRIBE="1"))
            assert run.returncode == 0, (row["name"], run.stdout, run.stderr)
            values = observations(run.stdout)
            assert values["forbidden_calls"] == "0" and values["created_mutexes"] == "1", (row["name"], values)
            recorded[row["name"]] = dict(primary_fixture=row, table_hex=row["table"], bits=row["bits"],
                omega_outcome=row["error"], omega_expected=expected(row), observed=values,
                stdout=run.stdout, stderr=run.stderr, exit_code=run.returncode,
                agrees_on_value_and_state=agrees(row, values))
        assert binary_hash == sha(binary)
    assert bound == snapshot() and all(sha(upstream / path) == value for path, value in original.items())
    record = dict(stage="Pinned public Interpreter load/evaluate; no native Omega or hardware claim",
        upstream_revision=PIN, upstream_source=str(upstream), upstream_sha256=original,
        source_sha256=bound, source_unchanged=True, binary=str(binary), binary_sha256=binary_hash,
        build_source=manifest, build_command=command, build_output=built.stdout + built.stderr,
        rustc=subprocess.check_output([str(rustc), "-Vv"], text=True),
        cargo=subprocess.check_output([str(cargo), "-V"], text=True), rows=recorded, omitted=omitted,
        omitted_primary_fixtures={row["name"]: row for row in fixtures.execution_cases() if row["name"] in omitted},
        agreements=sum(row["agrees_on_value_and_state"] for row in recorded.values()),
        nonagreements=sorted(name for name, row in recorded.items() if not row["agrees_on_value_and_state"]))
    args.record.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n")
    verify(json.loads(args.record.read_text()), True)


if __name__ == "__main__":
    main()
