"""Check real pane corners against one fill/stroke composite, not another pane painter."""
import argparse
from pathlib import Path
from dev import bitmap_rgb


def coverage(distance):
    return max(0, min(16, 8 - distance)) * 100 // 16


def corner_distance(horizontal, vertical, radius):
    horizontal = max(2 * horizontal - 1, 0)
    vertical = max(2 * vertical - 1, 0)
    larger, smaller = max(horizontal, vertical), min(horizontal, vertical)
    seed = max(larger * 8 + smaller + smaller // 2,
               larger * 7 + smaller * 4,
               larger * 6 + smaller * 5 + (larger + smaller) // 4)
    squared = (horizontal * horizontal + vertical * vertical) * 64
    return (seed + squared // seed) // 2 - radius * 16 if seed else -radius * 16


def check(path, left, top, right, radius):
    (width, height), channels = bitmap_rgb(path)
    assert 0 <= left < right <= width and 0 <= top < top + radius < height

    def pixel(horizontal, vertical):
        position = (height - 1 - vertical) * width + horizontal
        return tuple(channel[position] for channel in channels)

    outside = pixel(left - 2, top - 2)
    fill = pixel(left + radius + 4, top + 4)
    stroke = pixel(left + radius + 25, top)
    assert max(abs(first - second) for first, second in zip(outside, stroke)) > 8, path
    maximum = 0
    for vertical in range(radius):
        for horizontal in range(radius):
            distance = corner_distance(radius - horizontal, radius - vertical, radius)
            outer, inner = coverage(distance), coverage(distance + 16)
            expected = tuple((background * (100 - outer) + border * (outer - inner)
                              + surface * inner) // 100
                             for background, border, surface in zip(outside, stroke, fill))
            for column in (left + horizontal, right - 1 - horizontal):
                maximum = max(maximum, *(abs(actual - reference) for actual, reference
                                        in zip(pixel(column, top + vertical), expected)))
    print(f"{path.name}: maximum corner-channel error {maximum}; allowed 3")
    return maximum <= 3


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    arguments = parser.parse_args()
    # These are the fixed root-divider frame-20 geometries in the gallery replay.
    passed = [check(arguments.directory / f"panel_{scale}_0_{mode}_20.bmp", *geometry)
              for scale, geometry in (("d", (699, 205, 1117, 11)),
                                      ("4", (1864, 469, 2978, 26)))
              for mode in ("fast", "slow")]
    raise SystemExit(0 if all(passed) else 1)
