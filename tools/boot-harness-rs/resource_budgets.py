"""Validate boot/launch IPC reservations before building any executable.

These fixed lab limits mirror contracts/user.rs; kernel admission revalidates them.
"""
def channel_limits(startup):
    launches = startup.get("launches", [startup["launch"]] if startup.get("launch") else [])
    programs = [startup["initial"], *launches]
    if not 1 <= len(programs) <= 8:
        raise ValueError("IPC reservations require 1..8 boot/launch accounts")
    limits = [program.get("ipc_channels", 4) for program in programs]
    if any(type(limit) is not int or not 0 <= limit <= 8 for limit in limits):
        raise ValueError("ipc_channels must be an integer between 0 and 8")
    if sum(limits) > 32:
        raise ValueError("IPC channel reservations exceed the 32-channel backing pool")
    return limits
