; PortKiller Windows Installer Script
; Inno Setup 6.x required - Download from https://jrsoftware.org/isdl.php

#define MyAppName "PortKiller"
#define MyAppVersion "0.3.0-win"
#define MyAppPublisher "Samarth Gupta"
#define MyAppURL "https://github.com/gupsammy/PortKiller"
#define MyAppExeName "portkiller.exe"
#define MyAppId "{{B8F7E8A0-9C3D-4E5F-8A1B-2D3C4E5F6A7B}"

#define MyAppFileVersion "0.3.0.0"

[Setup]
; NOTE: The value of AppId uniquely identifies this application.
AppId={#MyAppId}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}/issues
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
LicenseFile=..\LICENSE
OutputDir=..\target\release\installer
OutputBaseFilename=PortKiller-{#MyAppVersion}-Setup
SetupIconFile=..\assets\app-icon.ico
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\assets\app-icon.ico
UninstallDisplayName={#MyAppName}
VersionInfoVersion={#MyAppFileVersion}
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription={#MyAppName} - Stop hunting. Start killing.
VersionInfoCopyright=Copyright (C) 2024 {#MyAppPublisher}
VersionInfoProductName={#MyAppName}
VersionInfoProductVersion={#MyAppFileVersion}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "startup"; Description: "Launch {#MyAppName} at Windows startup"; GroupDescription: "Startup Options:"; Flags: unchecked

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\app-logo-color.png"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "..\assets\app-icon.ico"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion isreadme
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon
Name: "{userstartup}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: startup

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#MyAppName}}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{userappdata}\PortKiller"

[Code]
function InitializeSetup(): Boolean;
var
  ResultCode: Integer;
  OldVersion: String;
begin
  Result := True;
  
  // Check if already installed
  if RegQueryStringValue(HKLM, 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{#MyAppId}_is1', 
                          'DisplayVersion', OldVersion) then
  begin
    if MsgBox('PortKiller ' + OldVersion + ' is already installed. Do you want to uninstall it first?', 
              mbConfirmation, MB_YESNO) = IDYES then
    begin
      // Uninstall old version
      if RegQueryStringValue(HKLM, 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{#MyAppId}_is1',
                              'UninstallString', OldVersion) then
      begin
        Exec(RemoveQuotes(OldVersion), '/SILENT', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
      end;
    end
    else
    begin
      Result := False;
    end;
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  ConfigPath: String;
  ConfigDir: String;
  LaunchAtLogin: String;
begin
  if CurStep = ssPostInstall then
  begin
    ConfigDir := ExpandConstant('{userappdata}\PortKiller');
    ConfigPath := ConfigDir + '\portkiller.json';
    
    // Check if startup task was selected
    if WizardIsTaskSelected('startup') then
      LaunchAtLogin := 'true'
    else
      LaunchAtLogin := 'false';

    // Create default config if it doesn't exist
    if not FileExists(ConfigPath) then
    begin
      // Ensure directory exists
      if not DirExists(ConfigDir) then
        CreateDir(ConfigDir);
        
      SaveStringToFile(ConfigPath, 
        '{' + #13#10 +
        '  "monitoring": {' + #13#10 +
        '    "poll_interval_secs": 2,' + #13#10 +
        '    "port_ranges": [[3000, 3010], [5432, 5432], [8080, 8090], [5000, 5010], [27017, 27017], [6379, 6379]],' + #13#10 +
        '    "show_project_names": true' + #13#10 +
        '  },' + #13#10 +
        '  "integrations": {' + #13#10 +
        '    "docker_enabled": true,' + #13#10 +
        '    "windows_services_enabled": true' + #13#10 +
        '  },' + #13#10 +
        '  "notifications": {' + #13#10 +
        '    "enabled": true' + #13#10 +
        '  },' + #13#10 +
        '  "system": {' + #13#10 +
        '    "launch_at_login": ' + LaunchAtLogin + #13#10 +
        '  }' + #13#10 +
        '}', False);
    end;
  end;
end;
