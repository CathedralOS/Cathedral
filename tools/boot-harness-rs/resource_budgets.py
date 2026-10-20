"""Validate boot/launch IPC backing and CPU ceilings before executable builds.

These fixed lab limits mirror kernel/contracts/user.rs; kernel admission revalidates them.
"""
def programs(startup):
    launches = startup.get("launches", [startup["launch"]] if startup.get("launch") else [])
    dynamic = startup.get("dynamic_tasks", 0)
    if type(dynamic) is not int or not 0 <= dynamic <= 30:
        raise ValueError("dynamic_tasks must be an integer in 0..30")
    entries = [startup["initial"], *launches, *({"ipc_channels": 0} for _ in range(dynamic))]
    if not 1 <= len(entries) <= 31:
        raise ValueError("resource budgets require 1..31 boot/launch accounts")
    return entries

def channel_limits(startup):
    limits = [program.get("ipc_channels", 4) for program in programs(startup)]
    if any(type(limit) is not int or not 0 <= limit <= 188 for limit in limits):
        raise ValueError("ipc_channels must be an integer between 0 and 188")
    if sum(limits) > 192:
        raise ValueError("IPC channel reservations exceed the 192-channel backing pool")
    return limits

def cpu_limits(startup):
    limits = [program.get("cpu_budget_us", 0) for program in programs(startup)]
    if any(type(limit) is not int or not 0 <= limit <= 100_000 for limit in limits):
        raise ValueError("cpu_budget_us must be an integer between 0 and 100000")
    # These are ceilings, not reservations: sums may exceed one CPU. Each child
    # is also constrained by every ancestor, even if its own field is zero.
    return limits
