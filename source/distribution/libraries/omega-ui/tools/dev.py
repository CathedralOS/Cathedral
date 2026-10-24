"""Build the compiler, verify its profile, then run one Omega UI development action.

We retain compiler output under build/ and propagate failed builds or package
reviews. The compiler checkout is supplied by the caller; no developer path is
part of the package. OMEGA_COMPILER_TARGET_DIR can reuse an existing build while
still checking that it matches the selected compiler source.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import struct
import sys


ROOT = Path(__file__).resolve().parents[1]
PROJECTS = {
    "gallery": ROOT / "examples" / "gallery",
    "headless_test": ROOT / "tests" / "headless",
}


def run(command, **options):
    print("+ " + subprocess.list2cmdline([str(value) for value in command]), flush=True)
    return subprocess.run(command, check=True, **options)


def verify_reference_execution(output):
    lines = set(output.splitlines())
    if not {"native exit 0", "interp exit: 0"} <= lines or any(
            line.startswith("interp: DECLINED") for line in lines):
        raise ValueError("Library verification requires successful native and Terminal reference execution")


def bitmap_rgb(path):
    data = path.read_bytes()
    if len(data) < 54 or data[:2] != b"BM":
        raise ValueError(f"Invalid BMP header: {path}")
    offset = struct.unpack_from("<I", data, 10)[0]
    width, height = struct.unpack_from("<ii", data, 18)
    planes, bits = struct.unpack_from("<HH", data, 26)
    compression = struct.unpack_from("<I", data, 30)[0]
    if width <= 0 or height <= 0 or (planes, bits, compression) != (1, 32, 0):
        raise ValueError(f"Expected bottom-up 32-bit BI_RGB: {path}")
    if offset < 54 or len(data) != offset + width * height * 4:
        raise ValueError(f"Invalid BMP pixel extent: {path}")
    pixels = data[offset:]
    # BI_RGB does not define the fourth byte as alpha; compare visible channels.
    return (width, height), (pixels[0::4], pixels[1::4], pixels[2::4])


def verify_bitmap_pair(direct, reference, size):
    direct_size, direct_rgb = bitmap_rgb(direct)
    reference_size, reference_rgb = bitmap_rgb(reference)
    if direct_size != size or reference_size != size or direct_rgb != reference_rgb:
        raise ValueError(f"Partial/full repaint mismatch: {direct.name} / {reference.name}")


def verify_overlay_background(background, overlay, rectangle):
    size, original = bitmap_rgb(background)
    overlay_size, painted = bitmap_rgb(overlay)
    left, top, right, bottom = rectangle
    width, height = size
    if overlay_size != size or not (0 <= left < right <= width and 0 <= top < bottom <= height):
        raise ValueError("Invalid overlay comparison geometry")
    for y in range(height):
        start = (height - 1 - y) * width
        spans = ((0, width),) if y < top or y >= bottom else ((0, left), (right, width))
        for x0, x1 in spans:
            for before, after in zip(original, painted):
                if before[start + x0:start + x1] != after[start + x0:start + x1]:
                    raise ValueError("Overlay changed background outside its bounds")
    return size


def snapshot_pairs():
    pairs = [(f"damage_direct_{index:02}.bmp", f"damage_upload_{index:02}.bmp", (1440, 945))
             for index in (0, 8, 16, 24, 30)]
    pairs += [(f"motion_direct_{index:02}.bmp", f"motion_upload_{index:02}.bmp", (1440, 945))
              for index in range(10)]
    pairs += [("snapshot_direct.bmp", "snapshot_upload.bmp", (1440, 945)),
              ("damage_4k_direct.bmp", "damage_4k_upload.bmp", (3840, 2160)),
              ("snapshot_4k_direct.bmp", "snapshot_4k_upload.bmp", (3840, 2160))]
    pairs += [(f"dot_{scale}_fast_{index:02}.bmp", f"dot_{scale}_slow_{index:02}.bmp", size)
              for scale, size in (("d", (1440, 945)), ("4", (3840, 2160)))
              for index in (1, 4)]
    pairs += [(f"search_{scale}_fast_{index}.bmp", f"search_{scale}_slow_{index}.bmp", size)
              for scale, size in (("d", (1440, 945)), ("4", (3840, 2160)))
              for index in (1, 2)]
    pairs += [(f"slider_frame_{index:02}.bmp", f"slider_full_{index:02}.bmp", (1440, 945))
              for index in (6, 20)]
    pairs += [(f"panel_{scale}_{kind}_fast_{index:02}.bmp", f"panel_{scale}_{kind}_slow_{index:02}.bmp", size)
              for scale, size in (("d", (1440, 945)), ("4", (3840, 2160)))
              for kind in range(3) for index in (6, 20, 24)]
    pairs += [(f"fallback_{scale}_good.bmp", f"fallback_{scale}_test.bmp", size)
              for scale, size in (("d", (1440, 945)), ("4", (3840, 2160)))]
    return pairs


def verify_snapshots(directory):
    pairs = snapshot_pairs()
    for direct, reference, size in pairs:
        verify_bitmap_pair(directory / direct, directory / reference, size)
    print(f"PASS: {len(pairs)} partial/full RGB comparisons at default and 4K sizes", flush=True)
    size = verify_overlay_background(directory / "snapshot_palette_background.bmp",
                                     directory / "snapshot_palette.bmp", (579, 154, 1210, 605))
    if size != (1440, 945):
        raise ValueError("Unexpected launcher comparison size")
    print("PASS: launcher preserves background labels outside its bounds", flush=True)
    return len(pairs)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("review", "resume", "test", "gallery", "snapshots"))
    parser.add_argument("--omega-repo", type=Path, default=os.environ.get("OMEGA_REPO"),
                        help="Omega source checkout (or set OMEGA_REPO)")
    parser.add_argument("--compiler-profile", choices=("release", "dev"), default="release",
                        help="compiler build profile (default: release)")
    parser.add_argument("--compiler-target-dir", type=Path,
                        default=os.environ.get("OMEGA_COMPILER_TARGET_DIR"),
                        help="reuse an Omega build directory; the compiler is rebuilt before use")
    parser.add_argument("--project", choices=("headless_test", "gallery"), default="gallery",
                        help="project for review/resume")
    arguments = parser.parse_args()
    if arguments.action in ("gallery", "snapshots") and os.name != "nt":
        parser.error("the gallery host requires Windows; use 'test' for the headless library suite")
    if arguments.omega_repo is None:
        parser.error("--omega-repo or OMEGA_REPO must name the compiler source checkout")
    repository = arguments.omega_repo.resolve()
    if not (repository / "omega-rust" / "omega" / "application" / "cli").is_dir():
        parser.error(f"not an Omega source checkout: {repository}")

    output = ROOT / "build"
    output.mkdir(exist_ok=True)
    build_directory = (arguments.compiler_target_dir or output / "compiler").resolve()
    build_log = output / "compiler-build.log"
    try:
        with build_log.open("w", encoding="utf-8") as log:
            command = [shutil.which("mbx") or "cargo", "build", "-p", "omega",
                       "--target-dir", build_directory]
            if arguments.compiler_profile == "release":
                command.append("--release")
            run(command, cwd=repository, stdout=log, stderr=subprocess.STDOUT)
    except subprocess.CalledProcessError:
        print(build_log.read_text(encoding="utf-8", errors="replace")[-8000:], file=sys.stderr)
        raise
    executable_name = "omega.exe" if os.name == "nt" else "omega"
    profile_directory = "release" if arguments.compiler_profile == "release" else "debug"
    compiler = build_directory / profile_directory / executable_name
    profile = json.loads(run([compiler, "--build-profile"], capture_output=True, text=True, encoding="utf-8", errors="replace").stdout)
    if profile.get("schema") != "omega-compiler-build-profile-v1" or profile.get("opt_level") not in ("1", "2", "3", "s", "z"):
        raise RuntimeError(f"compiler does not report an optimized build: {profile}")
    revision = run(["git", "rev-parse", "HEAD"], cwd=repository,
                   capture_output=True, text=True, encoding="utf-8", errors="replace").stdout.strip()
    dirty = run(["git", "status", "--porcelain"], cwd=repository,
                capture_output=True, text=True, encoding="utf-8", errors="replace").stdout
    receipt = {"compiler": str(compiler), "sha256": hashlib.sha256(compiler.read_bytes()).hexdigest(),
               "profile": profile, "cargo_profile": arguments.compiler_profile, "revision": revision, "working_tree_status": dirty,
               "built_at_utc": datetime.now(timezone.utc).isoformat()}
    (output / "compiler.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    print(f"Omega {revision[:12]}, opt-level {profile['opt_level']}, sha256 {receipt['sha256']}", flush=True)

    project = PROJECTS["headless_test" if arguments.action == "test" else arguments.project]
    if arguments.action in ("review", "resume"):
        command = [compiler, "update", "--offline"]
        if arguments.action == "resume":
            command.append("--resume")
        run(command, cwd=project)
    elif arguments.action == "test":
        try:
            result = run([compiler, "run", "--both", "main.omg"], cwd=project,
                         stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                         text=True, encoding="utf-8", errors="replace")
        except subprocess.CalledProcessError as error:
            print(error.stdout or "", end="", flush=True)
            raise
        print(result.stdout, end="", flush=True)
        verify_reference_execution(result.stdout)
    else:
        project = PROJECTS["gallery"]
        destination = output / "gallery"
        run([compiler, "--timings", "--build-dir", destination, "main.omg"], cwd=project)
        shutil.copytree(ROOT / "assets" / "fonts" / "inter", destination / "fonts", dirs_exist_ok=True)
        if arguments.action == "snapshots":
            environment = dict(os.environ, OMEGA_HEADLESS="1")
            # A Windows GUI image needs explicit inherited output handles even
            # when this helper's own output is redirected to a log file.
            command = [destination / "omega-program.exe"]
            for direct, reference, _ in snapshot_pairs():
                (destination / direct).unlink(missing_ok=True)
                (destination / reference).unlink(missing_ok=True)
            for name in ("snapshot_palette.bmp", "snapshot_palette_background.bmp"):
                (destination / name).unlink(missing_ok=True)
            print("+ " + subprocess.list2cmdline([str(value) for value in command]), flush=True)
            result = subprocess.run(command, cwd=destination, env=environment,
                                    capture_output=True, text=True, encoding="utf-8", errors="replace")
            print(result.stdout, end="", flush=True)
            print(result.stderr, end="", file=sys.stderr, flush=True)
            result.check_returncode()
            verify_snapshots(destination)


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as error:
        sys.exit(error.returncode)
