// use headersgen::generate_headers_bindings;
use engine::vulkan_mod;

fn main() {
    // generate_headers_bindings();
    let _ = vulkan_mod::load_vulkan_lib();
    let _ = vulkan_mod::get_vulkan_available_extensions();
    println!("Done Loading");
}
