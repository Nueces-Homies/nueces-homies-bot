pub mod api {
    include!(concat!(env!("OUT_DIR"), "/igdb.rs"));
}

pub mod client;
pub mod errors;
pub mod query;
