// =============================================================================
// Helper Functions
// =============================================================================
//
// Shared utility functions used across the server module.
//
// Modules:
// - auth.rs       : JWT parsing, user profile extraction
// - validation.rs : Input validation (names, messages, etc.)
//
// Future helpers:
// - combat.rs     : Damage calculations, type effectiveness
// - pathfinding.rs: Enemy path calculations
// - spawning.rs   : Wave and enemy spawning logic
//
// =============================================================================

pub mod auth;
pub mod validation;
