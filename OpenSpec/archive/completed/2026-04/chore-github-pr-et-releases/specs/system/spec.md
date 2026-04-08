# Specs Delta: GitHub PR et Releases

## ADDED Requirements

### Requirement: Protected Main Branch
- **GIVEN** the FutureAero repository is maintained on GitHub
- **WHEN** changes target `main`
- **THEN** the canonical branch must be protected by pull-request based integration and explicit required checks.

### Requirement: Required CI Checks Stay Explicit
- **GIVEN** a pull request is opened against `main`
- **WHEN** GitHub evaluates the branch protections
- **THEN** Rust, frontend, desktop shell and coverage checks must be listed explicitly and remain aligned with the versioned workflow configuration.

### Requirement: Desktop Installer Is Published By CI
- **GIVEN** the GitHub Actions workflow completes successfully
- **WHEN** the desktop packaging job finishes
- **THEN** a Windows installer artifact must be attached to the workflow run and available for download.
