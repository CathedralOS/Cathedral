"""Validate boot/launch IPC backing and CPU ceilings before executable builds.

These fixed lab limits mirror contracts/user.rs; kernel admission revalidates them.
"""
def programs(startup):
    launches = startup.get("launches", [startup["launch"]] if startup.get("launch") else [])
    entries = [startup["initial"], *launches]
    if not 1 <= len(entries) <= 8:
        raise ValueError("resource budgets require 1..8 boot/launch accounts")
    return entries

def channel_limits(startup):
    limits = [program.get("ipc_channels", 4) for program in programs(startup)]
    if any(type(limit) is not int or not 0 <= limit <= 8 for limit in limits):
        raise ValueError("ipc_channels must be an integer between 0 and 8")
    if sum(limits) > 32:
        raise ValueError("IPC channel reservations exceed the 32-channel backing pool")
    return limits

def cpu_limits(startup):
    limits = [program.get("cpu_budget_us", 0) for program in programs(startup)]
    if any(type(limit) is not int or not 0 <= limit <= 100_000 for limit in limits):
        raise ValueError("cpu_budget_us must be an integer between 0 and 100000")
    # These are ceilings, not reservations: sums may exceed one CPU. Each child
    # is also constrained by every ancestor, even if its own field is zero.
    return limits
