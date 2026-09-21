#!/usr/bin/env python3
"""Run focused additional Program graph witnesses with the existing checked driver."""
import importlib.util
import sys
from pathlib import Path
import witnesses

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
spec = importlib.util.spec_from_file_location('boundary_check', HERE.parent/'check.py')
driver = importlib.util.module_from_spec(spec)
spec.loader.exec_module(driver)
driver.HERE = HERE
driver.fixtures = witnesses
original_snapshot = driver.snapshot


def snapshot():
    inputs = original_snapshot()
    # The reused renderer and driver are also execution inputs.
    for path in [HERE.parent/'fixtures.py', HERE.parent/'check.py']:
        inputs[str(path.relative_to(driver.ROOT))] = driver.sha(path)
    return dict(sorted(inputs.items()))


driver.snapshot = snapshot
if __name__ == '__main__':
    driver.main()
