#!/usr/bin/env python3
"""Audit exact integration source composition; this is not execution evidence."""
import argparse
import hashlib
import json
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
BASE_REVISION = "3f33097"
CONCAT_REVISION = "3c9753b"
WRITE_PIPELINE_REVISION = "9e75b27"
WRITE_RETIREMENT_REVISION = "20f1e02"
TO_STRING_REVISION = "21ef7e2"
LOGICAL_SOURCE_REVISION = "c87658c"
LOGICAL_REVISION = "2acc896"
SOURCE = "source/libraries/acpi/"
EXECUTION = SOURCE + "interpreter/execution/"
TOOLS = "tools/ports/acpi/"

WHOLE_FILES = {
    CONCAT_REVISION: (
        SOURCE + "aml/inline_concat.omg",
        *(EXECUTION + name + ".omg" for name in (
            "concat_execution", "engine", "execution_model", "runtime_model", "write_retirement"
        )),
    ),
    WRITE_PIPELINE_REVISION: tuple(SOURCE + "pipeline/" + name + ".omg"
                                   for name in ("build", "field_writes", "program")),
    TO_STRING_REVISION: (EXECUTION + "to_string_execution.omg",),
    LOGICAL_SOURCE_REVISION: (EXECUTION + "logical_execution.omg",),
}
OWNED_DIRECTORIES = {
    TOOLS + "interpreter/mid-execution": BASE_REVISION,
    TOOLS + "interpreter/concat-execution": CONCAT_REVISION,
    TOOLS + "interpreter/field-write-execution": WRITE_RETIREMENT_REVISION,
    TOOLS + "pipeline/field-writes": WRITE_PIPELINE_REVISION,
    TOOLS + "interpreter/to-string-execution": TO_STRING_REVISION,
    TOOLS + "interpreter/logical-execution": LOGICAL_REVISION,
}
BORROWED_INPUTS = tuple(TOOLS + name for name in (
    "interpreter/execution/fixtures.py",
    "interpreter/execution/checked_runner.rs",
    "interpreter/execution/runner.Cargo.lock",
    "interpreter/generic-execution/fixtures.py",
    "interpreter/generic-execution/decoder_fixtures.py",
    "interpreter/generic-execution/focused/bridge-atomicity/main.omg",
    "interpreter/named-store-execution/fixtures.py",
    "interpreter/to-integer-execution/fixtures.py",
    "pipeline/fixtures.py",
    "aml-public-execution/src/main.rs",
    "aml-public-execution/Cargo.toml",
    "aml-public-execution/Cargo.lock",
))


def _git(*args, input=None):
    return subprocess.run(["git", *args], cwd=ROOT, input=input,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True).stdout


def _sha(data):
    return hashlib.sha256(data).hexdigest()


def _tree(commit):
    result = {}
    for item in _git("ls-tree", "-r", "-z", commit).split(b"\0"):
        if not item:
            continue
        metadata, path = item.split(b"\t", 1)
        mode, kind, blob = metadata.decode().split()
        result[path.decode()] = dict(mode=mode, kind=kind, blob=blob)
    return result


def _blobs(ids):
    ids = sorted(set(ids))
    stream = _git("cat-file", "--batch", input="".join(blob + "\n" for blob in ids).encode())
    result, at = {}, 0
    for wanted in ids:
        end = stream.index(b"\n", at)
        actual, kind, size = stream[at:end].decode().split()
        if actual != wanted or kind != "blob":
            raise ValueError("unexpected git cat-file response")
        at, length = end + 1, int(size)
        result[wanted] = stream[at:at + length]
        at += length
        if stream[at:at + 1] != b"\n":
            raise ValueError("unterminated git blob response")
        at += 1
    if at != len(stream):
        raise ValueError("unconsumed git blob response")
    return result


def _tool_input(path, directory):
    relative = Path(path).relative_to(directory)
    # Historical diagnostic/evidence packages and regenerated JSON receipts
    # remain independently attributed artifacts, outside this input audit.
    if any(part in {"diagnostics", "evidence", "__pycache__"} for part in relative.parts):
        return False
    return (relative.suffix == ".py" or relative.name.endswith("cases.json")
            or relative.name in {"toolchain.json", "rejection-helpers.omg"})


def _edits():
    """Insertion-only union relative to the retained Concatenate source."""
    retire = EXECUTION + "retire.omg"
    specs = EXECUTION + "operator_specs.omg"
    changes = []

    def insert(path, owner, anchor, added, after=False):
        changes.append(dict(path=path, revision=owner, old=anchor,
                            new=anchor + added if after else added + anchor,
                            inserted=added))

    insert(retire, TO_STRING_REVISION, "use concat_execution::retire_concat;\n",
           "use to_string_execution::retire_to_string;\n", after=True)
    insert(retire, LOGICAL_SOURCE_REVISION, "use mid_execution::retire_mid;\n",
           "use logical_execution::retire_logical;\n", after=True)
    insert(retire, TO_STRING_REVISION,
           "  0x99 -> integer_conversion_retirement(input,space,frame,operation.operands[0],operation.first_target)\n",
           "  0x9c -> string_conversion_retirement(input,space,frame,operation)\n", after=True)
    insert(retire, LOGICAL_SOURCE_REVISION,
           "  0x9d -> copy(input,space,frame,operation.first_target,operation.operands[0],true)\n",
           "".join(f"  {opcode} -> logical_retirement(input,space,frame,operation)\n"
                   for opcode in ("0x90", "0x91", "0x92", "0x93", "0x94", "0x95", "0x9293", "0x9294", "0x9295")),
           after=True)
    anchor = " state mid_retirement(input:&[u8;1024],space:&mut ObjectStore,frame:&mut Frame,operation:Operation)->ExecutionOutcome {\n"
    insert(retire, TO_STRING_REVISION, anchor,
           " state string_conversion_retirement(input:&[u8;1024],space:&mut ObjectStore,frame:&mut Frame,operation:Operation)->ExecutionOutcome {\n"
           "  let result:ExecutionOutcome=retire_to_string(input,space,frame,operation.operands[0],operation.operands[1],operation.first_target);result\n }\n")
    insert(retire, LOGICAL_SOURCE_REVISION, anchor,
           " state logical_retirement(input:&[u8;1024],space:&ObjectStore,frame:&mut Frame,operation:Operation)->ExecutionOutcome {\n"
           "  let result:ExecutionOutcome=retire_logical(input,space,frame,operation.opcode,operation.operands[0],operation.operands[1]);result\n }\n")
    for old, added in (
        ("        0x99 -> (1) 0x9d -> (1) 0x9e -> (3) 0xa0 -> (1) 0xa2 -> (1) 0xa4 -> (1)\n", "0x9c -> (2) "),
        ("        0x99 -> (1) 0x9d -> (1) 0x9e -> (1) 0x5b28 -> (1) 0x5b29 -> (1)\n", "0x9c -> (1) "),
    ):
        changes.append(dict(path=specs, revision=TO_STRING_REVISION, old=old,
                            new=old.replace("0x9d", added + "0x9d", 1), inserted=added))
    return changes


def run():
    """Return a successful source-only audit or raise on any mismatch."""
    refs = {BASE_REVISION, CONCAT_REVISION, WRITE_PIPELINE_REVISION,
            WRITE_RETIREMENT_REVISION, TO_STRING_REVISION,
            LOGICAL_SOURCE_REVISION, LOGICAL_REVISION}
    revisions = {ref: _git("rev-parse", "--verify", ref + "^{commit}").decode().strip()
                 for ref in sorted(refs)}
    trees = {ref: _tree(commit) for ref, commit in revisions.items()}

    def entry(ref, path):
        found = trees[ref].get(path)
        if found is None or found["kind"] != "blob" or found["mode"] not in {"100644", "100755"}:
            raise ValueError(f"not a regular source blob: {ref}:{path}")
        return found

    production = {path: BASE_REVISION for path in trees[BASE_REVISION]
                  if path.startswith(SOURCE) and path.endswith(".omg")}
    for ref, paths in WHOLE_FILES.items():
        for path in paths:
            entry(ref, path)
            production[path] = ref
    for name in ("retire", "operator_specs"):
        production[EXECUTION + name + ".omg"] = CONCAT_REVISION
    actual_source = {str(path.relative_to(ROOT)) for path in (ROOT / SOURCE).rglob("*.omg")}
    if actual_source != set(production):
        raise ValueError(f"ACPI source set differs: missing={sorted(set(production)-actual_source)}, unexpected={sorted(actual_source-set(production))}")

    fixture = {path: BASE_REVISION for path in BORROWED_INPUTS}
    directory_sets = {}
    for directory, ref in OWNED_DIRECTORIES.items():
        wanted = {path for path in trees[ref]
                  if path.startswith(directory + "/") and _tool_input(path, directory)}
        actual = {str(path.relative_to(ROOT)) for path in (ROOT / directory).rglob("*")
                  if (path.is_file() or path.is_symlink())
                  and _tool_input(str(path.relative_to(ROOT)), directory)}
        if wanted != actual:
            raise ValueError(f"fixture/tool set differs in {directory}: missing={sorted(wanted-actual)}, unexpected={sorted(actual-wanted)}")
        directory_sets[directory] = dict(commit=revisions[ref], paths=sorted(wanted))
        for path in wanted:
            fixture[path] = ref

    requested = {(ref, path) for path, ref in {**production, **fixture}.items()}
    requested |= {(change["revision"], change["path"]) for change in _edits()}
    blobs = _blobs(entry(ref, path)["blob"] for ref, path in requested)

    def original(ref, path):
        return blobs[entry(ref, path)["blob"]]

    expected = {path: original(ref, path) for path, ref in production.items()}
    unions = {}
    for change in _edits():
        path, ref = change["path"], change["revision"]
        old, new, added = (change[key].encode() for key in ("old", "new", "inserted"))
        if new.replace(added, b"", 1) != old:
            raise ValueError("union transformation is not an insertion")
        if original(ref, path).count(added) != 1:
            raise ValueError(f"insertion is not uniquely present in owner: {ref}:{path}")
        if expected[path].count(old) != 1 or added in expected[path]:
            raise ValueError(f"union anchor or insertion multiplicity differs: {path}")
        expected[path] = expected[path].replace(old, new, 1)
        unions.setdefault(path, []).append(dict(
            source_commit=revisions[ref], source_blob=entry(ref, path)["blob"],
            source_sha256=_sha(original(ref, path)), old=change["old"],
            new=change["new"], inserted=change["inserted"],
        ))

    failures = []

    def compare(path, ref, wanted, edits=None):
        current = ROOT / path
        if not current.is_file() or current.is_symlink():
            raise ValueError(f"current input is not a regular file: {path}")
        actual = current.read_bytes()
        if actual != wanted:
            failures.append(path)
        result = dict(expected_sha256=_sha(wanted), actual_sha256=_sha(actual),
                      equal=actual == wanted, origin_commit=revisions[ref],
                      origin_blob=entry(ref, path)["blob"], origin_sha256=_sha(original(ref, path)),
                      construction="explicit insertion union" if edits else "whole file")
        if edits:
            result["insertions"] = edits
        return result

    source_files = {path: compare(path, ref, expected[path], unions.get(path))
                    for path, ref in sorted(production.items())}
    fixture_files = {path: compare(path, ref, original(ref, path))
                     for path, ref in sorted(fixture.items())}
    if failures:
        raise ValueError("exact source/fixture bytes differ: " + ", ".join(failures))
    return dict(
        audit="exact-source-composition", verified=True, execution_validation=False,
        scope="All recursive ACPI .omg sources plus selected historical tool/fixture inputs; no compilation, interpretation or public replay is asserted.",
        root=str(ROOT), head=_git("rev-parse", "HEAD").decode().strip(),
        revisions=revisions, auditor_sha256=_sha(Path(__file__).read_bytes()),
        source_file_set=sorted(production), source_files=source_files,
        fixture_directories=directory_sets, fixture_files=fixture_files,
        borrowed_inputs=list(BORROWED_INPUTS),
        exclusions="README/docs, diagnostic/evidence archives and generated receipts (including public/verification.json); public/check.py sources remain bound.",
        source_count=len(source_files), fixture_count=len(fixture_files),
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--record", type=Path)
    args = parser.parse_args()
    record = run()
    if args.record:
        args.record.write_text(json.dumps(record, indent=2) + "\n")
    print(f"PASS source audit only: {record['source_count']} ACPI sources, {record['fixture_count']} fixture/tool inputs; execution not tested.")


if __name__ == "__main__":
    main()
