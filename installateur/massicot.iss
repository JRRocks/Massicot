; Windows installer of Massicot (Inno Setup 6 or 7), built by the CI: scripts/installateur.sh.
;   ISCC /DVersion=0.1.12 /DSource=<package folder> /DSortie=<output folder> massicot.iss
; Per-user install without administrator rights, always in %LOCALAPPDATA%\Programs\Massicot (Claude
; Code's MCP configuration points there); a newer installer installs over the previous one.
; Choices validated on previews (DECISIONS.md, D-22): three pages, icon 1. The Start menu shortcut
; opens the window (D-27), through massicot-fenetre.exe, which starts it without a console.

#ifndef Version
  #error Version manquante : ISCC /DVersion=0.1.12 ...
#endif
#ifndef Source
  #error Dossier du paquet manquant : ISCC /DSource=paquet\Massicot.<version> ...
#endif
#ifndef Sortie
  #define Sortie ".."
#endif
; Never change it: it identifies the installed application (updates, uninstall entry).
#define AppGuid "753EA851-ACAE-40AE-99C9-B23F7633F159"

[Setup]
AppId={{{#AppGuid}}
AppName=Massicot
AppVersion={#Version}
AppVerName=Massicot {#Version}
AppPublisher=Massicot
AppPublisherURL=https://github.com/JRRocks/Massicot
AppSupportURL=https://github.com/JRRocks/Massicot
AppUpdatesURL=https://github.com/JRRocks/Massicot/releases
AppComments=Boîte à outils PDF
AppCopyright=Licence MIT ou Apache-2.0
VersionInfoVersion={#Version}
VersionInfoProductName=Massicot
VersionInfoDescription=Installation de Massicot
VersionInfoCompany=Massicot
DefaultDirName={localappdata}\Programs\Massicot
UsePreviousAppDir=no
DisableDirPage=yes
DisableProgramGroupPage=yes
DisableWelcomePage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
ChangesEnvironment=yes
; Claude Code may be running massicot.exe (MCP server): it is renamed, not closed (see [Code]).
CloseApplications=no
RestartApplications=no
SetupIconFile=..\assets\icone\massicot.ico
UninstallDisplayIcon={app}\massicot.exe
UninstallDisplayName=Massicot
WizardStyle=modern
WizardImageFile=..\assets\icone\assistant-grand-164x314.bmp,..\assets\icone\assistant-grand-246x459.bmp,..\assets\icone\assistant-grand-328x628.bmp,..\assets\icone\assistant-grand-410x797.bmp
WizardSmallImageFile=..\assets\icone\assistant-petit-55x55.bmp,..\assets\icone\assistant-petit-83x80.bmp,..\assets\icone\assistant-petit-110x106.bmp,..\assets\icone\assistant-petit-138x140.bmp
OutputDir={#Sortie}
OutputBaseFilename=Massicot.{#Version}
Compression=lzma2/max
SolidCompression=yes
SetupLogging=yes
ShowLanguageDialog=no

[Languages]
Name: "fr"; MessagesFile: "compiler:Languages\French.isl"

[Messages]
SetupWindowTitle=Installation - %1
FinishedLabel=[name/ver] est installé.%n%nVous le trouverez dans le menu Démarrer, et la commande massicot marche dans tout nouveau terminal.
FinishedLabelNoIcons=[name/ver] est installé.%n%nVous le trouverez dans le menu Démarrer, et la commande massicot marche dans tout nouveau terminal.
ClickFinish=Cliquez sur Terminer pour quitter l'assistant.
; The ready page is the first one: there is no Back button to mention.
ReadyLabel2a=Cliquez sur Installer pour procéder à l'installation.
UninstallAppTitle=Désinstallation de Massicot
UninstallAppFullTitle=Désinstallation de %1
ConfirmUninstall=Voulez-vous vraiment désinstaller complètement %1 ainsi que tous ses composants ?
UninstalledAll=%1 a été correctement désinstallé de cet ordinateur.%n%nVos PDF et la configuration de Claude Code n'ont pas été touchés ; pensez à retirer le serveur « massicot » de Claude Code si vous ne réinstallez pas.

[Files]
; The same files as the zip: an install over a copy unzipped by hand takes all of them over.
Source: "{#Source}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
; The window of Massicot (massicot.exe fenetre, started without a console by massicot-fenetre.exe).
Name: "{autoprograms}\Massicot"; Filename: "{app}\massicot-fenetre.exe"; WorkingDir: "{userdocs}"; IconFilename: "{app}\massicot.exe"; Comment: "Boîte à outils PDF"

[Run]
Filename: "{app}\massicot-fenetre.exe"; WorkingDir: "{userdocs}"; Description: "Ouvrir Massicot"; Flags: postinstall nowait skipifsilent

[Code]
const
  CleDesinstallation = 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{{#AppGuid}}_is1';
  CleEnvironnement = 'Environment';

var
  VersionInstallee: String; { version installed by this installer before, or '' }
  ExeMisDeCote: String;     { previous massicot.exe, renamed before the files are installed }

function InitializeSetup(): Boolean;
begin
  if not RegQueryStringValue(HKCU, CleDesinstallation, 'DisplayVersion', VersionInstallee) then
    VersionInstallee := '';
  Result := True;
end;

procedure InitializeWizard();
begin
  WizardForm.Caption := 'Installation - Massicot'; { as on the validated preview, without the version }
end;

{ --- Pages --- }

procedure CurPageChanged(CurPageID: Integer);
begin
  if (CurPageID = wpReady) and (VersionInstallee <> '') then
  begin
    WizardForm.PageNameLabel.Caption := 'Prêt à mettre à jour';
    if VersionInstallee = '{#Version}' then
      WizardForm.PageDescriptionLabel.Caption := Format('Massicot %s est déjà installé sur cet ordinateur. L''assistant va le réinstaller.', [VersionInstallee])
    else
      WizardForm.PageDescriptionLabel.Caption := Format('Massicot %s est installé sur cet ordinateur. L''assistant va le remplacer par la version %s.', [VersionInstallee, '{#Version}']);
    WizardForm.ReadyLabel.Caption := 'Cliquez sur Mettre à jour. Rien à désinstaller avant.';
    WizardForm.NextButton.Caption := 'Mettre à jour';
  end;
end;

function UpdateReadyMemo(Space, NewLine, MemoUserInfoInfo, MemoDirInfo, MemoTypeInfo, MemoComponentsInfo, MemoGroupInfo, MemoTasksInfo: String): String;
var
  Dossier: String;
begin
  Dossier := ExpandConstant('{app}');
  if VersionInstallee = '' then
    Result := 'Massicot {#Version}, boîte à outils PDF' + NewLine + NewLine +
      'Dossier d''installation :' + NewLine + Space + Dossier + NewLine + NewLine +
      'Menu Démarrer :' + NewLine + Space + 'Massicot' + NewLine + NewLine +
      'Commande « massicot » :' + NewLine + Space + 'utilisable dans tout nouveau terminal (ajoutée au PATH)'
  else
    Result := 'Version installée :  ' + VersionInstallee + NewLine +
      'Nouvelle version :   {#Version}' + NewLine + NewLine +
      'Dossier (inchangé) :' + NewLine + Space + Dossier + NewLine + NewLine +
      'Vos réglages et la configuration de Claude Code sont gardés.' + NewLine +
      'Si Claude Code utilise Massicot en ce moment, il passera' + NewLine +
      'à la nouvelle version à son prochain démarrage.';
end;

{ --- PATH of the user (HKCU\Environment) --- }

function MemeDossier(const Element, Dossier: String): Boolean;
var
  E: String;
begin
  E := Trim(Element);
  StringChangeEx(E, '%LOCALAPPDATA%', ExpandConstant('{localappdata}'), True);
  StringChangeEx(E, '%USERPROFILE%', ExpandConstant('{%USERPROFILE}'), True);
  Result := (E <> '') and (CompareText(RemoveBackslashUnlessRoot(E), RemoveBackslashUnlessRoot(Dossier)) = 0);
end;

{ Adds the folder to the user's PATH (once), or removes it, leaving the other entries as they are:
  removing gives back the PATH exactly as it was before the install. }
procedure ModifierPath(const Dossier: String; Ajouter: Boolean);
var
  Ancien, Reste, Element, Nouveau: String;
  P: Integer;
  Present, Premier: Boolean;
begin
  if not RegQueryStringValue(HKCU, CleEnvironnement, 'Path', Ancien) then
    Ancien := '';
  Nouveau := '';
  Present := False;
  Premier := True;
  if Ancien <> '' then
    Reste := Ancien + ';'
  else
    Reste := '';
  while Reste <> '' do
  begin
    P := Pos(';', Reste);
    Element := Copy(Reste, 1, P - 1);
    Delete(Reste, 1, P);
    if MemeDossier(Element, Dossier) then
      Present := True
    else
    begin
      if not Premier then
        Nouveau := Nouveau + ';';
      Nouveau := Nouveau + Element;
      Premier := False;
    end;
  end;
  if Ajouter then
  begin
    if Present then
      Exit;
    if Premier then
      Nouveau := Dossier
    else if Copy(Ancien, Length(Ancien), 1) = ';' then
      Nouveau := Nouveau + Dossier + ';' { keep the trailing ";" last, as Windows writes it }
    else
      Nouveau := Nouveau + ';' + Dossier;
  end
  else if not Present then
    Exit;
  if Nouveau = '' then
    RegDeleteValue(HKCU, CleEnvironnement, 'Path')
  else
    RegWriteExpandStringValue(HKCU, CleEnvironnement, 'Path', Nouveau);
end;

{ --- massicot.exe in use (Claude Code's MCP server) --- }

{ A running program can be neither replaced nor deleted, but it can be renamed: the running server
  goes on with the old version, and the new file takes the name. }
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  Exe, Nom: String;
  I: Integer;
begin
  Result := '';
  Exe := ExpandConstant('{app}\massicot.exe');
  if not FileExists(Exe) then
    Exit;
  I := 1;
  repeat
    Nom := Exe + '.ancien-' + IntToStr(I);
    I := I + 1;
  until not FileExists(Nom);
  if RenameFile(Exe, Nom) then
    ExeMisDeCote := Nom;
end;

{ Deletes the renamed old versions no longer running; at uninstall, moves the others to the
  temporary folder so that nothing is left in the install folder. }
procedure SupprimerAnciens(VersTemp: Boolean);
var
  R: TFindRec;
  Chemin: String;
begin
  if FindFirst(ExpandConstant('{app}\massicot.exe.ancien-*'), R) then
  try
    repeat
      Chemin := ExpandConstant('{app}\') + R.Name;
      if not DeleteFile(Chemin) and VersTemp then
        RenameFile(Chemin, AddBackslash(GetTempDir()) + 'massicot-' + GetDateTimeString('yyyymmddhhnnsszzz', #0, #0) + '-' + R.Name);
    until not FindNext(R);
  finally
    FindClose(R);
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    ModifierPath(ExpandConstant('{app}'), True);
    ExeMisDeCote := '';
    SupprimerAnciens(False);
  end;
end;

{ Install cancelled or failed after the rename: the old massicot.exe gets its name back. }
procedure DeinitializeSetup();
var
  Exe: String;
begin
  if ExeMisDeCote <> '' then
  begin
    Exe := ExpandConstant('{app}\massicot.exe');
    if FileExists(ExeMisDeCote) and not FileExists(Exe) then
      RenameFile(ExeMisDeCote, Exe);
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Exe: String;
begin
  if CurUninstallStep = usUninstall then
  begin
    SupprimerAnciens(True);
    Exe := ExpandConstant('{app}\massicot.exe');
    if FileExists(Exe) and not DeleteFile(Exe) then
      RenameFile(Exe, AddBackslash(GetTempDir()) + 'massicot-' + GetDateTimeString('yyyymmddhhnnsszzz', #0, #0) + '.exe');
  end
  else if CurUninstallStep = usPostUninstall then
  begin
    ModifierPath(ExpandConstant('{app}'), False);
    { Folders that existed before the install (a copy unzipped by hand) are not in the uninstall log. }
    RemoveDir(ExpandConstant('{app}\modeles'));
    RemoveDir(ExpandConstant('{app}\licences'));
    RemoveDir(ExpandConstant('{app}'));
  end;
end;
