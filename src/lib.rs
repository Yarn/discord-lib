
#![recursion_limit="1024"]
// #![feature(async_await, futures_api, pin)]
// #![feature(impl_trait_in_bindings)]

// extern crate futures;
// extern crate futures01;

// extern crate hyper;
// extern crate hyper_tls;
// extern crate tokio;
// extern crate tokio_tungstenite;
#[macro_use] extern crate failure;
// extern crate bytes;

// extern crate serde;
#[macro_use] extern crate serde_derive;
// extern crate serde_json;

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

// pub use futures;
// pub use tokio;
// pub use hyper;
// pub use serde_json;
// pub use bytes;
