# Wrapper — real script lives in tools/scripts/install/
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
& "$Root\tools\scripts\install\install.ps1" @args
