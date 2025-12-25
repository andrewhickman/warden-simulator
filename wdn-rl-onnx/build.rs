fn main() {
    let descriptors = protox::compile(["onnx.proto3"], ["onnx/onnx"]).unwrap();
    prost_build::Config::new()
        .bytes([".onnx"])
        .compile_fds(descriptors)
        .unwrap();
}
