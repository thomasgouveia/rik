fn main() {
    tonic_build::compile_protos("../proto/controller.proto").unwrap();
    tonic_build::compile_protos("../proto/worker.proto").unwrap();
}
