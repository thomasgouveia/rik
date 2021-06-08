fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../proto/controller.proto")?;
    tonic_build::compile_protos("google/protobuf/empty.proto")?;
    tonic_build::compile_protos("../proto/worker.proto")?;
    Ok(())
}