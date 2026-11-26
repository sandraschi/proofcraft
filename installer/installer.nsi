; Current-user NSIS installer for proofcraft (L2 CLI/MCP + L3 egui studio)
!define APP_NAME "proofcraft"
!define APP_VERSION "0.1.0"
!define PUBLISHER "sandraschi fleet"

Name "${APP_NAME} ${APP_VERSION}"
OutFile "..\dist\proofcraft-setup.exe"
InstallDir "$LOCALAPPDATA\Programs\${APP_NAME}"
RequestExecutionLevel user

Page directory
Page instfiles

Section "Install"
  SetOutPath "$INSTDIR"
  File "..\dist\proofcraft-app.exe"
  File "..\dist\proofcraft-mcp.exe"
  File "..\README.md"
  File "..\HELP.md"
  CreateDirectory "$SMPROGRAMS\${APP_NAME}"
  CreateShortCut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\proofcraft-app.exe"
  CreateShortCut "$SMPROGRAMS\${APP_NAME}\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\proofcraft-app.exe"
  Delete "$INSTDIR\proofcraft-mcp.exe"
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\HELP.md"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"
  Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
  Delete "$SMPROGRAMS\${APP_NAME}\Uninstall.lnk"
  RMDir "$SMPROGRAMS\${APP_NAME}"
SectionEnd
