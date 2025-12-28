pub use prost;
pub mod minibear {
    pub mod schema {
        include!(concat!(env!("OUT_DIR"), "/minibear.schema.rs"));
    }
}
