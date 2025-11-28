!include "MUI2.nsh"
!include "FileFunc.nsh"

Name "Prometheus CLI"
OutFile "prometheus-windows-installer.exe"
InstallDir "$LOCALAPPDATA\Prometheus"
RequestExecutionLevel user

!define MUI_ABORTWARNING
!define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "Install"
    SetOutPath "$INSTDIR"

    File "prometheus-cli.exe"
    File "config.toml"
    File "README.md"
    File /r "docs"

    CreateDirectory "$INSTDIR\conversations"
    CreateDirectory "$APPDATA\prometheus"

    IfFileExists "$APPDATA\prometheus\config.toml" +2 0
    CopyFiles "$INSTDIR\config.toml" "$APPDATA\prometheus\config.toml"

    WriteUninstaller "$INSTDIR\Uninstall.exe"

    CreateDirectory "$SMPROGRAMS\Prometheus"
    CreateShortcut "$SMPROGRAMS\Prometheus\Prometheus CLI.lnk" "$INSTDIR\prometheus-cli.exe"
    CreateShortcut "$SMPROGRAMS\Prometheus\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
    CreateShortcut "$DESKTOP\Prometheus CLI.lnk" "$INSTDIR\prometheus-cli.exe"

    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Prometheus" "DisplayName" "Prometheus CLI"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Prometheus" "UninstallString" "$INSTDIR\Uninstall.exe"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Prometheus" "DisplayVersion" "__VERSION__"

    EnVar::AddValue "PATH" "$INSTDIR"

    MessageBox MB_YESNO "Prometheus CLI has been installed.$\n$\nDo you want to install dependencies (Rust and Ollama)?" IDYES install_deps IDNO skip_deps

    install_deps:
        DetailPrint "Installing Rust..."
        NSISdl::download "https://win.rustup.rs/x86_64" "$TEMP\rustup-init.exe"
        ExecWait '"$TEMP\rustup-init.exe" -y'
        Delete "$TEMP\rustup-init.exe"

        DetailPrint "Installing Ollama..."
        NSISdl::download "https://ollama.com/download/OllamaSetup.exe" "$TEMP\OllamaSetup.exe"
        ExecWait '"$TEMP\OllamaSetup.exe" /S'
        Delete "$TEMP\OllamaSetup.exe"

        MessageBox MB_OK "Dependencies installed. Please restart your terminal."

    skip_deps:
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\prometheus-cli.exe"
    Delete "$INSTDIR\config.toml"
    Delete "$INSTDIR\README.md"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir /r "$INSTDIR\docs"
    RMDir "$INSTDIR"

    Delete "$SMPROGRAMS\Prometheus\Prometheus CLI.lnk"
    Delete "$SMPROGRAMS\Prometheus\Uninstall.lnk"
    RMDir "$SMPROGRAMS\Prometheus"
    Delete "$DESKTOP\Prometheus CLI.lnk"

    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Prometheus"

    EnVar::DeleteValue "PATH" "$INSTDIR"
SectionEnd
