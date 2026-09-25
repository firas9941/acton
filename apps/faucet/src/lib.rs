mod antifraud;
mod app;
mod auth;
mod blockchain;
mod handlers;
pub mod middlewares;

pub use app::run;

pub(crate) const LONG_VERSION: &str = env!("FAUCET_LONG_VERSION");
