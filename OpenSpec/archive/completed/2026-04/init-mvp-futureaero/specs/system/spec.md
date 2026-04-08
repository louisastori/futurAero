# Specs Delta: FutureAero Core System

## ADDED Requirements

### Requirement: Desktop Architecture (Local First)
- **GIVEN** an engineering user working offline
- **WHEN** they launch FutureAero
- **THEN** the application must run entirely locally without requiring cloud dependencies to protect IP.

### Requirement: White-Box Transparency
- **GIVEN** a generated mechanical constraint or AI suggestion
- **WHEN** the user inspects the property panel
- **THEN** the mathematical relationships, kinematics chains, and AI decision sources must be visible, editable, and explainable.

### Requirement: Local AI Assistance
- **GIVEN** a complex modeling task
- **WHEN** the user queries the AI chat panel
- **THEN** the local LLM (via Ollama) must provide context-aware suggestions based on the current `.faero` project state without modifying the file silently.