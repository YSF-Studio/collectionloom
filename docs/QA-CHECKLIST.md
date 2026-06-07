# CollectionLoom QA Checklist

Use this checklist to verify the portable app on macOS, Windows, and Linux.

## 1. App shell

- Launch the portable binary without installing it.
- Confirm the window title, theme toggle, and sidebar render correctly.
- Switch between light, dark, and system theme.
- Confirm the status bar stays responsive.

## 2. Prerequisites

- Open **Prerequisites**.
- Confirm the app shows:
  - bundled vs source-built mode
  - portable kit root, if any
  - missing tools
  - tools found on PATH
  - privilege warnings when applicable
- Re-run the check and confirm the summary updates.

## 3. Disk Imaging

- Open **Disk Imaging**.
- Confirm the source selector, destination field, hash options, and split input render.
- Confirm **Start Acquisition** stays disabled until a source is selected.
- Refresh the disk list.
- Select a disk and confirm write-blocker controls become meaningful.
- Confirm the destination path defaults to a portable-safe output folder.

## 4. RAM Capture

- Open **RAM Capture**.
- Confirm the tool selector is visible.
- Confirm the output path is prefilled.
- Confirm the capture button remains disabled until a valid tool is available.
- Confirm the process list button opens a process list.

## 5. Mobile, Network, Cloud, and Case tabs

- Open each tab once and confirm the UI renders without errors.
- Confirm status hints explain missing binaries or permissions clearly.
- Confirm any action buttons are disabled when prerequisites are missing.

## 6. Export and Chain of Custody

- Open **Custody Chain** and create a record if test data is available.
- Open **Export Bundle** and confirm export options render.
- Confirm folder-opening actions work when case data exists.

## 7. Fixture/demo mode

- Use fixture mode for UI-only verification when real tools or devices are not available.
- Confirm the app still behaves predictably when binaries are missing.
- Use this mode to test layouts, warnings, and navigation on every platform.

## Pass criteria

- No crash on launch.
- Sidebar navigation works.
- Disabled states match the available tools.
- Warnings are specific and actionable.
- Portable paths stay inside the kit when portable mode is enabled.
