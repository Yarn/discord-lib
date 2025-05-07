
#![recursion_limit="1024"]
#![allow(non_local_definitions)] // silence warning originating from failure macros

#[macro_use] extern crate failure;
#[macro_use] extern crate serde_derive;

pub mod gateway;
pub mod gateway_ws;
pub mod discord;
pub mod send_message;
pub(crate) mod set_reaction;
pub(crate) mod discord_api;
mod outer_wrapper;
pub use self::outer_wrapper::Discord;
pub use self::outer_wrapper::SendHandle;
pub use discord_api::channel::Channel;

pub use self::gateway_ws::jank_run;
