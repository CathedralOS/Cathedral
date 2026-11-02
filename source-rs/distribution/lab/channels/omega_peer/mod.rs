mod endpoints;
#[cfg(feature = "omega-pipeline")]
mod pipeline;
mod service;
mod services;
mod supervisor;
include!("peer.rs");
