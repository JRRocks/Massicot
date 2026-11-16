# Screenshots of the installer's real pages on the CI runner (Windows PowerShell 5.1), so that the
# user can compare them with the validated previews: first install, Start menu console, uninstall,
# update. Saved as PNG in DIR. Not a test: the CI goes on if the runner shows no screen.
param(
    [Parameter(Mandatory)] [string] $Nouveau,   # installer to publish
    [Parameter(Mandatory)] [string] $Ancien,    # installer of the previous release
    [Parameter(Mandatory)] [string] $Dossier,   # where the PNG files go
    [Parameter(Mandatory)] [string] $Pdf        # an invented PDF to show in the window
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms, System.Drawing, UIAutomationClient, UIAutomationTypes
Add-Type @"
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class Fenetres {
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    delegate bool Rappel(IntPtr h, IntPtr l);
    [DllImport("user32.dll")] static extern bool EnumWindows(Rappel f, IntPtr l);
    [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h, int a, out RECT r, int n);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
    // Visible windows whose title starts with (or is) the text.
    public static IntPtr[] Trouver(string titre, bool exact) {
        var liste = new List<IntPtr>();
        EnumWindows((h, l) => {
            if (!IsWindowVisible(h)) return true;
            var s = new StringBuilder(512);
            GetWindowText(h, s, 512);
            var t = s.ToString();
            RECT r;
            GetWindowRect(h, out r);
            if ((exact ? t == titre : t.StartsWith(titre)) && r.Right - r.Left > 150 && r.Bottom - r.Top > 80) liste.Add(h);
            return true;
        }, IntPtr.Zero);
        return liste.ToArray();
    }
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassName(IntPtr h, StringBuilder s, int n);
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    // Every visible window: title, class, process, size (diagnostics).
    public static string[] Lister() {
        var liste = new List<string>();
        EnumWindows((h, l) => {
            if (!IsWindowVisible(h)) return true;
            var t = new StringBuilder(512); GetWindowText(h, t, 512);
            var c = new StringBuilder(256); GetClassName(h, c, 256);
            uint pid; GetWindowThreadProcessId(h, out pid);
            RECT r; GetWindowRect(h, out r);
            liste.Add(string.Format("{0} | {1} | pid {2} | {3}x{4}", t, c, pid, r.Right - r.Left, r.Bottom - r.Top));
            return true;
        }, IntPtr.Zero);
        return liste.ToArray();
    }
}
"@
New-Item -ItemType Directory -Force $Dossier | Out-Null
$Nouveau = (Resolve-Path $Nouveau).Path
$Ancien = (Resolve-Path $Ancien).Path
$App = Join-Path $env:LOCALAPPDATA 'Programs\Massicot'
$UIA = [System.Windows.Automation.AutomationElement]

# A button of the window, by one of its names (message box buttons follow the language of Windows).
# Inno Setup's buttons are seen as panes by UI Automation: found by name only.
function Bouton([IntPtr] $H, [string[]] $Noms) {
    foreach ($nom in $Noms) {
        $cond = New-Object System.Windows.Automation.PropertyCondition($UIA::NameProperty, $nom)
        $b = $UIA::FromHandle($H).FindFirst([System.Windows.Automation.TreeScope]::Descendants, $cond)
        if ($b -and $b.Current.NativeWindowHandle -ne 0) { return $b }
    }
}

# The window with this title that shows this button (a page or a message box).
function Attendre([string] $Titre, [string[]] $NomBouton, [switch] $Exact, [int] $Secondes = 40) {
    $limite = (Get-Date).AddSeconds($Secondes)
    while ((Get-Date) -lt $limite) {
        foreach ($h in [Fenetres]::Trouver($Titre, [bool]$Exact)) {
            if (-not $NomBouton) { return $h }
            try { if (Bouton $h $NomBouton) { return $h } } catch { }
        }
        Start-Sleep -Milliseconds 300
    }
    Write-Host "Fenêtres visibles :"
    [Fenetres]::Lister() | ForEach-Object { Write-Host "  $_" }
    foreach ($h in [Fenetres]::Trouver($Titre, [bool]$Exact)) {
        Write-Host "Boutons de « $Titre » :"
        $tous = $UIA::FromHandle($h).FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
        foreach ($e in $tous) { Write-Host "  $($e.Current.ControlType.ProgrammaticName) « $($e.Current.Name) »" }
    }
    Get-Process | Where-Object { $_.ProcessName -like 'Massicot*' } | ForEach-Object { Write-Host "Processus $($_.ProcessName) ($($_.Id)) fenêtre « $($_.MainWindowTitle) »" }
    CapturerEcran 'echec-ecran-complet'
    throw "fenêtre « $Titre » avec le bouton « $($NomBouton -join ' / ') » introuvable"
}

# BM_CLICK, posted: the click works whatever window has the focus.
function Cliquer([IntPtr] $H, [string[]] $NomBouton) {
    $b = Bouton $H $NomBouton
    [Fenetres]::PostMessage([IntPtr] $b.Current.NativeWindowHandle, 0x00F5, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
}

function Capturer([IntPtr] $H, [string] $Nom) {
    [Fenetres]::SetForegroundWindow($H) | Out-Null
    Start-Sleep -Milliseconds 800
    $r = New-Object Fenetres+RECT
    if ([Fenetres]::DwmGetWindowAttribute($H, 9, [ref] $r, 16) -ne 0) { [Fenetres]::GetWindowRect($H, [ref] $r) | Out-Null }
    $l = $r.Right - $r.Left; $h = $r.Bottom - $r.Top
    $image = New-Object System.Drawing.Bitmap $l, $h
    $g = [System.Drawing.Graphics]::FromImage($image)
    $g.CopyFromScreen($r.Left, $r.Top, 0, 0, $image.Size)
    $g.Dispose()
    $image.Save((Join-Path $Dossier "$Nom.png"), [System.Drawing.Imaging.ImageFormat]::Png)
    $image.Dispose()
    Write-Host "capture : $Nom ($l x $h)"
}

function CapturerEcran([string] $Nom) {
    $b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    $image = New-Object System.Drawing.Bitmap $b.Width, $b.Height
    $g = [System.Drawing.Graphics]::FromImage($image)
    $g.CopyFromScreen($b.Left, $b.Top, 0, 0, $image.Size)
    $g.Dispose()
    $image.Save((Join-Path $Dossier "$Nom.png"), [System.Drawing.Imaging.ImageFormat]::Png)
    $image.Dispose()
}

# Consoles opened by the shortcut or "Ouvrir Massicot": only their PowerShell is stopped (on the
# runner, the console may open as a tab of the Terminal window that runs the CI job itself).
function FermerConsoles {
    Get-CimInstance Win32_Process -Filter "Name = 'powershell.exe'" |
        Where-Object { $_.CommandLine -like '*massicot.exe*accueil*' } |
        ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
}

# The window, opened as the Start menu shortcut does (massicot-fenetre.exe), with an invented PDF.
# Optional: a runner without a graphics adapter may not open it.
function CapturerFenetre([string] $Pdf) {
    try {
        Start-Process -FilePath (Join-Path $App 'massicot-fenetre.exe') -ArgumentList "`"$Pdf`"" | Out-Null
        $f = Attendre 'Massicot' -Exact -Secondes 40
        Start-Sleep -Seconds 6
        Capturer $f '4-fenetre'
    } catch {
        Write-Host "Fenêtre non capturée : $_"
    } finally {
        Get-Process massicot -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    }
}

function Silencieux([string] $Exe, [string[]] $Arguments) {
    $p = Start-Process -FilePath $Exe -ArgumentList ($Arguments + '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART') -PassThru
    if (-not $p.WaitForExit(120000)) { throw "$Exe ne se termine pas" }
    Start-Sleep -Seconds 3
}

# Off main, the PNG files also go to the job log (base64), readable where artifacts are not.
function Journaliser {
    if ($env:GITHUB_REF -eq 'refs/heads/main') { return }
    foreach ($f in Get-ChildItem $Dossier -Filter *.png) {
        Write-Host "CAPTURE-DEBUT $($f.Name)"
        $b64 = [Convert]::ToBase64String([IO.File]::ReadAllBytes($f.FullName))
        for ($i = 0; $i -lt $b64.Length; $i += 2000) { Write-Host ('CAPTURE ' + $b64.Substring($i, [Math]::Min(2000, $b64.Length - $i))) }
        Write-Host "CAPTURE-FIN $($f.Name)"
    }
}

$ecran = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
Write-Host "Écran du runner : $($ecran.Width) x $($ecran.Height)"

try {
# 1. First install, then the window ("Ouvrir Massicot" is ticked; the window opened by the
#    installer is closed, then opened again on an invented PDF).
$p = Start-Process -FilePath $Nouveau -PassThru
$w = Attendre 'Installation - Massicot' 'Installer'
Capturer $w '1-pret-a-installer'
Cliquer $w 'Installer'
$w = Attendre 'Installation - Massicot' 'Terminer'
Capturer $w '3-termine'
Cliquer $w 'Terminer'
Start-Sleep -Seconds 4
Get-Process massicot -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
$p.WaitForExit(60000) | Out-Null
CapturerFenetre (Resolve-Path $Pdf).Path

# 2. Uninstall: confirmation, then the last message.
Start-Process -FilePath (Join-Path $App 'unins000.exe')
$d = Attendre 'Désinstallation' 'Oui', 'Yes'
Capturer $d '5-desinstallation-confirmation'
Cliquer $d 'Oui', 'Yes'
$d = Attendre 'Désinstallation' 'OK'
Capturer $d '6-desinstallation-terminee'
Cliquer $d 'OK'
Start-Sleep -Seconds 3

# 3. Update over the previous release.
Silencieux $Ancien @()
$p = Start-Process -FilePath $Nouveau -PassThru
$w = Attendre 'Installation - Massicot' 'Mettre à jour'
Capturer $w '2-pret-a-mettre-a-jour'
Cliquer $w 'Mettre à jour'
$w = Attendre 'Installation - Massicot' 'Terminer'
Cliquer $w 'Terminer'
Start-Sleep -Seconds 4
Get-Process massicot -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
$p.WaitForExit(60000) | Out-Null
Silencieux (Join-Path $App 'unins000.exe') @()
Write-Host 'Captures faites.'
} finally {
    Journaliser
}

