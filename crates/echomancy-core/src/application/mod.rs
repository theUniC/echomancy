//! Application layer — thin orchestration over the `Game` aggregate.
//!
//! Provides:
//! - **Commands** (`commands`) — mutate state via the repository.
//! - **Queries** (`queries`) — read state from the repository.
//! - **Errors** (`errors`) — `ApplicationError` enum.
//! - **Repository** (`repository`) — `GameRepository` trait.

pub mod commands;
pub mod errors;
pub mod queries;
pub mod repository;
pub(crate) mod validation;
