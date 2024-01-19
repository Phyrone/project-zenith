/// === Common ===
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Position {
    #[prost(sint64, tag = "1")]
    pub x: i64,
    #[prost(sint64, tag = "2")]
    pub y: i64,
    #[prost(sint64, tag = "3")]
    pub z: i64,
    #[prost(double, optional, tag = "17")]
    pub fine_x: ::core::option::Option<f64>,
    #[prost(double, optional, tag = "18")]
    pub fine_y: ::core::option::Option<f64>,
    #[prost(double, optional, tag = "19")]
    pub fine_z: ::core::option::Option<f64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Velocity {
    #[prost(double, tag = "1")]
    pub x: f64,
    #[prost(double, tag = "2")]
    pub y: f64,
    #[prost(double, tag = "3")]
    pub z: f64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Rotation {
    #[prost(float, tag = "1")]
    pub pitch: f32,
    #[prost(float, tag = "2")]
    pub yaw: f32,
}
