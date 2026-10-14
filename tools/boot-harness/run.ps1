# Shared Python owner keeps the Windows and POSIX boot paths identical.
$ErrorActionPreference = "Stop"
& python (Join-Path $PSScriptRoot "run.py") @args
exit $LASTEXITCODE
