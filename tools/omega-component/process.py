"""Bound compiler memory before starting it, including any child processes.

Windows job limits are enforced by the kernel, rather than a polling watchdog
which can miss a rapid allocation spike. A separate Python process joins the
job so neither the caller nor other compiler sessions enter it.
"""
import ctypes
import os
from pathlib import Path
import subprocess
import sys

COMPILER_MEMORY = 4 * 1024**3
COMPILER_TIMEOUT = 300


def run(command, **kwargs):
    try:
        return subprocess.run([sys.executable, str(Path(__file__).resolve()),
                               str(COMPILER_MEMORY), *map(str, command)],
                              timeout=COMPILER_TIMEOUT, **kwargs)
    except subprocess.TimeoutExpired as error:
        raise RuntimeError(f'compiler exceeded {COMPILER_TIMEOUT}s; stopped its process group') from error


def constrain(memory):
    if os.name != 'nt':
        import resource
        resource.setrlimit(resource.RLIMIT_AS, (memory, memory))
        return None

    from ctypes import wintypes as w
    size = ctypes.c_size_t

    class Basic(ctypes.Structure):
        _fields_ = [('process_time', ctypes.c_int64), ('job_time', ctypes.c_int64),
                    ('flags', w.DWORD), ('minimum', size), ('maximum', size),
                    ('active', w.DWORD), ('affinity', size), ('priority', w.DWORD),
                    ('scheduling', w.DWORD)]

    class Io(ctypes.Structure):
        _fields_ = [(name, ctypes.c_uint64) for name in
                    ('read_ops', 'write_ops', 'other_ops', 'read_bytes', 'write_bytes', 'other_bytes')]

    class Extended(ctypes.Structure):
        _fields_ = [('basic', Basic), ('io', Io), ('process_memory', size),
                    ('job_memory', size), ('peak_process', size), ('peak_job', size)]

    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.CreateJobObjectW.argtypes = [ctypes.c_void_p, w.LPCWSTR]
    kernel.CreateJobObjectW.restype = w.HANDLE
    kernel.SetInformationJobObject.argtypes = [w.HANDLE, ctypes.c_int, ctypes.c_void_p, w.DWORD]
    kernel.SetInformationJobObject.restype = w.BOOL
    kernel.AssignProcessToJobObject.argtypes = [w.HANDLE, w.HANDLE]
    kernel.AssignProcessToJobObject.restype = w.BOOL
    kernel.GetCurrentProcess.restype = w.HANDLE
    job = kernel.CreateJobObjectW(None, None)
    if not job:
        raise ctypes.WinError(ctypes.get_last_error())
    limits = Extended()
    limits.basic.flags = 0x200 | 0x2000  # JOB_MEMORY | KILL_ON_JOB_CLOSE
    limits.job_memory = memory
    if not kernel.SetInformationJobObject(job, 9, ctypes.byref(limits), ctypes.sizeof(limits)):
        raise ctypes.WinError(ctypes.get_last_error())
    if not kernel.AssignProcessToJobObject(job, kernel.GetCurrentProcess()):
        raise ctypes.WinError(ctypes.get_last_error())
    # Keep the non-inheritable handle open until this wrapper exits. If the
    # caller times out and terminates us, Windows also terminates the compiler.
    return job


if __name__ == '__main__':
    held_job = constrain(int(sys.argv[1]))
    completed = subprocess.run(sys.argv[2:])
    if completed.returncode:
        print(f'compiler command exited {completed.returncode}; memory limit {int(sys.argv[1]) // 1024**2} MiB',
              file=sys.stderr)
    sys.exit(completed.returncode)
