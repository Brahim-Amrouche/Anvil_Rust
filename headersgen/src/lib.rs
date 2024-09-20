use bindgen::Builder;
use std::path::PathBuf;

pub fn generate_headers_bindings()
{
    let include_path = String::from("-I") + &std::env::var("VULKAN_SDK").unwrap() + "/include";

    let bindings = Builder::default()
        .header("wrapper.h")
        .clang_arg(include_path)
        .generate_comments(true)
        .generate()
        .expect("Couldn't generate the bindings");

    let output_buff = PathBuf::from("./engine/bindings");
    bindings
        .write_to_file(output_buff.join("binding.rs"))
        .expect("Good Bindings happened");
}
