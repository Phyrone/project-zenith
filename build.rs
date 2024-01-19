use std::error::Error;

use prost_build::Config;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=proto/*.proto");
    println!("cargo:rerun-if-changed=src/protocol/*.rs");

    Config::new()
        .out_dir("src/protocol")
        .format(true)
        .compile_protos(
            &[
                "proto/common.proto",
                "proto/packets.proto",
                "proto/datagram.proto",
                "proto/error.proto",
                "proto/channel.proto",
            ],
            &["proto/"],
        )?;

    Ok(())
}
