; Tauri NSIS hooks. This file is !included before the template's defines and vars, so everything that
; reads them (MAINBINARYSRCPATH, $UpdateMode, $DeleteAppDataCheckboxState) lives inside the macros.

Var PeepThisUnhooked

; The statusline wrapper is a second cargo bin, built next to the main exe by beforeBuildCommand.
; The app copies it from $INSTDIR to %APPDATA%\PeepThis\bin, the stable path Claude Code's statusLine points at.
!macro NSIS_HOOK_POSTINSTALL
  !searchreplace PEEPTHIS_STATUSLINE_SRC "${MAINBINARYSRCPATH}" "${MAINBINARYNAME}.exe" "peepthis-statusline.exe"
  File "/oname=$INSTDIR\peepthis-statusline.exe" "${PEEPTHIS_STATUSLINE_SRC}"
!macroend

; Upgrades run the old uninstaller with /UPDATE: leave the integration alone then.
!macro NSIS_HOOK_PREUNINSTALL
  StrCpy $PeepThisUnhooked 0
  ${If} $UpdateMode <> 1
    MessageBox MB_YESNO|MB_ICONQUESTION "Also remove Peep This from Claude Code?$\r$\n$\r$\nThis takes out its hooks and restores your original statusline. Your settings.json is backed up first." /SD IDNO IDNO peepthis_keep
    ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --uninstall-integration' $0
    ${If} $0 == 0
      StrCpy $PeepThisUnhooked 1
    ${Else}
      MessageBox MB_OKCANCEL|MB_ICONEXCLAMATION "Peep This could not remove its Claude Code integration, so settings.json is unchanged.$\r$\n$\r$\nOK uninstalls anyway. Cancel stops here, so you can use Uninstall integration in the tray menu first." /SD IDOK IDOK peepthis_keep
      Abort
    ${EndIf}
    peepthis_keep:
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  Delete "$INSTDIR\peepthis-statusline.exe"
  RMDir "$INSTDIR"
  ; %APPDATA%\PeepThis holds the wrapper copy a kept statusLine still runs, so only clear it once unhooked.
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
  ${AndIf} $PeepThisUnhooked = 1
    SetShellVarContext current
    RmDir /r "$APPDATA\PeepThis"
  ${EndIf}
!macroend
