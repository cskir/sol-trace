pub mod client;
pub mod server;
pub mod state;

pub mod proto {
    tonic::include_proto!("cli");
}
