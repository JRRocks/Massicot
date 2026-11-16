# Test of the Windows installer on the CI runner (PowerShell 7), before anything is published.
#   1. A copy of the previous release unzipped by hand (as on the PC), the new installer over it:
#      shortcut, uninstall entry (HKCU), PATH, massicot --version; silent uninstall, nothing left.
#   2. The previous release's installer, then the new one over it while an MCP server started from
#      the install folder is running (as Claude Code does); uninstall while a server is running;
#      nothing left.
param(
    [Parameter(Mandatory)] [string] $Nouveau,          # installer to publish
    [Parameter(Mandatory)] [string] $Version,          # its version
    [Parameter(Mandatory)] [string] $Ancien,           # installer of the previous release
    [Parameter(Mandatory)] [string] $AncienneVersion,  # its version
    [Parameter(Mandatory)] [string] $CopieManuelle     # unzipped package of the previous release
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName System.Drawing

$App = Join-Path $env:LOCALAPPDATA 'Programs\Massicot'
$Exe = Join-Path $App 'massicot.exe'
$Raccourci = Join-Path ([Environment]::GetFolderPath('Programs')) 'Massicot.lnk'
$Desinstallation = 'Software\Microsoft\Windows\CurrentVersion\Uninstall'
$Cle = "HKCU:\$Desinstallation\{753EA851-ACAE-40AE-99C9-B23F7633F159}_is1"
$Journaux = Join-Path ([IO.Path]::GetTempPath()) 'massicot-essai-installateur'
New-Item -ItemType Directory -Force $Journaux | Out-Null
$Nouveau = (Resolve-Path $Nouveau).Path
$Ancien = (Resolve-Path $Ancien).Path

function Verifier([bool] $Condition, [string] $Quoi) {
    if (-not $Condition) { throw "ÉCHEC : $Quoi" }
    Write-Host "  ok : $Quoi"
}

function PathUtilisateur { [Environment]::GetEnvironmentVariable('Path', 'User') }

function Installer([string] $Installateur, [string] $Nom) {
    $log = Join-Path $Journaux "$Nom.log"
    Write-Host "Installation silencieuse : $(Split-Path -Leaf $Installateur)"
    $p = Start-Process -FilePath $Installateur -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', "/LOG=`"$log`"" -Wait -PassThru
    if ($p.ExitCode -ne 0) {
        if (Test-Path $log) { Get-Content $log -Tail 60 }
        throw "ÉCHEC : installation $Nom, code $($p.ExitCode)"
    }
}

function Desinstaller([string] $Nom) {
    $unins = Join-Path $App 'unins000.exe'
    Verifier (Test-Path $unins) 'désinstalleur présent'
    $log = Join-Path $Journaux "desinstallation-$Nom.log"
    Write-Host 'Désinstallation silencieuse'
    Start-Process -FilePath $unins -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', "/LOG=`"$log`"" -Wait | Out-Null
    # The uninstaller goes on from a copy of itself in the temporary folder: wait for it.
    $limite = (Get-Date).AddMinutes(2)
    while (((Test-Path $Cle) -or (Test-Path $App)) -and (Get-Date) -lt $limite) { Start-Sleep -Milliseconds 500 }
    if (Test-Path $log) { Get-Content $log -Tail 15 }
}

function EntreesMassicot {
    @(Get-ChildItem "HKCU:\$Desinstallation" | Get-ItemProperty |
        Where-Object { $_.PSObject.Properties['DisplayName'] -and $_.DisplayName -eq 'Massicot' })
}

function VerifierInstalle([string] $V, [switch] $Complet) {
    $sortie = (& $Exe --version | Out-String).Trim()
    Verifier ($sortie -eq "massicot $V") "massicot --version depuis le dossier installé : $sortie"
    Verifier (Test-Path $Cle) 'entrée de désinstallation dans HKCU'
    $e = Get-ItemProperty $Cle
    Verifier ($e.DisplayVersion -eq $V) "version affichée dans Applications installées : $($e.DisplayVersion)"
    Verifier (@(EntreesMassicot).Count -eq 1) 'une seule entrée « Massicot » dans Applications installées'
    Verifier (-not (Test-Path "HKLM:\$Desinstallation\{753EA851-ACAE-40AE-99C9-B23F7633F159}_is1")) 'rien dans HKLM (installation sans droits administrateur)'
    if (-not $Complet) { return }

    Verifier ($e.DisplayName -eq 'Massicot') "nom : $($e.DisplayName)"
    Verifier ($e.Publisher -eq 'Massicot') "éditeur : $($e.Publisher)"
    Verifier ($e.DisplayIcon -like '*\Programs\Massicot\massicot.exe') "icône : $($e.DisplayIcon)"
    Verifier ($e.UninstallString -like '*\Programs\Massicot\unins000.exe*') "désinstalleur : $($e.UninstallString)"
    Verifier ($e.InstallLocation.TrimEnd('\') -eq $App) "dossier : $($e.InstallLocation)"
    Verifier (Test-Path (Join-Path $App 'modeles\text-recognition.onnx')) 'modèles OCR installés'
    $infos = (Get-Item $Exe).VersionInfo
    Verifier ($infos.ProductName -eq 'Massicot' -and $infos.ProductVersion -eq $V) "ressources de massicot.exe (icône, version) : $($infos.ProductName) $($infos.ProductVersion)"
    Verifier ($null -ne [System.Drawing.Icon]::ExtractIcon($Exe, 0, 32)) 'icône intégrée à massicot.exe'

    Verifier (Test-Path $Raccourci) "raccourci du menu Démarrer : $Raccourci"
    $lnk = (New-Object -ComObject WScript.Shell).CreateShortcut($Raccourci)
    Verifier ($lnk.TargetPath -like '*\WindowsPowerShell\v1.0\powershell.exe') "il lance PowerShell : $($lnk.TargetPath)"
    Verifier ($lnk.Arguments -like "*-NoExit*& '$Exe' accueil*") "avec l'accueil de Massicot : $($lnk.Arguments)"
    Verifier ($lnk.IconLocation -like "$Exe,*") "son icône : $($lnk.IconLocation)"
    # The shortcut's command, without -NoExit so that it ends.
    $sortieConsole = Join-Path $Journaux 'console.txt'
    $p = Start-Process -FilePath $lnk.TargetPath -ArgumentList ($lnk.Arguments -replace '-NoExit ', '') -WorkingDirectory $lnk.WorkingDirectory `
        -RedirectStandardOutput $sortieConsole -NoNewWindow -Wait -PassThru
    $texte = Get-Content $sortieConsole -Raw
    Verifier ($p.ExitCode -eq 0 -and $texte -match "^Massicot $([regex]::Escape($V)) ") "la console du raccourci affiche l'accueil : $(($texte -split "`n")[0])"

    $dansPath = @((PathUtilisateur) -split ';' | Where-Object { $_.Trim().TrimEnd('\') -eq $App })
    Verifier ($dansPath.Count -eq 1) 'dossier présent une fois dans le PATH de l''utilisateur'
    # A new terminal: PATH read from the registry, as Windows gives it to a new process.
    $pathAvant = $env:Path
    try {
        $env:Path = [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' + (PathUtilisateur)
        $trouve = (Get-Command massicot -CommandType Application | Select-Object -First 1).Source
        Verifier ($trouve -eq $Exe) "« massicot » trouvé par le PATH : $trouve"
        $sortie = (& massicot --version | Out-String).Trim()
        Verifier ($sortie -eq "massicot $V") "massicot --version dans un nouveau terminal : $sortie"
    } finally {
        $env:Path = $pathAvant
    }
}

function VerifierRienNeReste($PathAvant) {
    Verifier (-not (Test-Path $App)) "dossier supprimé : $App"
    Verifier (-not (Test-Path $Raccourci)) 'raccourci du menu Démarrer supprimé'
    Verifier (-not (Test-Path $Cle)) 'entrée de désinstallation supprimée'
    Verifier (@(EntreesMassicot).Count -eq 0) 'plus rien dans Applications installées'
    Verifier ([string](PathUtilisateur) -eq [string]$PathAvant) 'PATH de l''utilisateur rendu tel qu''avant'
}

# An MCP server started from the install folder, kept waiting on its input, as Claude Code does.
function DemarrerMcp {
    $psi = [Diagnostics.ProcessStartInfo]::new($Exe)
    foreach ($a in 'mcp', '--racine', $Journaux) { $psi.ArgumentList.Add($a) }
    $psi.UseShellExecute = $false
    $psi.RedirectStandardInput = $true
    $psi.RedirectStandardOutput = $true
    $p = [Diagnostics.Process]::Start($psi)
    Start-Sleep -Seconds 1
    Verifier (-not $p.HasExited) 'serveur MCP lancé depuis le dossier installé'
    $p
}

function ArreterMcp([Diagnostics.Process] $P) {
    $P.StandardInput.Close()
    if (-not $P.WaitForExit(10000)) { $P.Kill() }
}

Verifier (-not (Test-Path $App) -and -not (Test-Path $Cle)) 'aucun Massicot installé au départ'
$pathDepart = PathUtilisateur

Write-Host "`n== 1. Copie $AncienneVersion décompressée à la main, puis installateur $Version par-dessus"
New-Item -ItemType Directory -Force $App | Out-Null
Copy-Item -Recurse -Path (Join-Path $CopieManuelle '*') -Destination $App
Verifier ((& $Exe --version | Out-String).Trim() -eq "massicot $AncienneVersion") "copie à la main en place ($AncienneVersion)"
Installer $Nouveau 'sur-copie-manuelle'
VerifierInstalle $Version -Complet
Desinstaller 'sur-copie-manuelle'
VerifierRienNeReste $pathDepart

Write-Host "`n== 2. Installateur $AncienneVersion, puis $Version par-dessus pendant que Claude Code utilise Massicot"
Installer $Ancien 'ancienne-version'
VerifierInstalle $AncienneVersion
$mcp = DemarrerMcp
Installer $Nouveau 'mise-a-jour'
VerifierInstalle $Version -Complet
Verifier (-not $mcp.HasExited) 'le serveur MCP lancé avant la mise à jour tourne toujours'
$mcp.StandardInput.WriteLine('{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"essai","version":"1"}}}')
$reponse = $mcp.StandardOutput.ReadLineAsync()
Verifier ($reponse.Wait(10000) -and $reponse.Result -like '*"serverInfo"*') 'et il répond encore'
ArreterMcp $mcp
Write-Host "Contenu du dossier après la mise à jour :"
Get-ChildItem $App | Format-Table Name, Length -AutoSize | Out-String | Write-Host

Write-Host "`n== 3. Désinstallation pendant qu'un serveur MCP tourne"
$mcp = DemarrerMcp
Desinstaller 'mise-a-jour'
VerifierRienNeReste $pathDepart
ArreterMcp $mcp

Write-Host "`nEssai de l'installateur réussi."
