Start-Sleep -Seconds 5
$wshell = New-Object -ComObject Wscript.Shell;
$wshell.SendKeys("^{ENTER}")
Write-Host "Sent Ctrl+Enter to active window"
