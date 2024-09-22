use engine::vulkan_mod;

macro_rules! INIT_OPERATION_UNWRAP_RESULT {
    ($func_res: expr) => {
        match $func_res {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    };
}

fn main() {
    INIT_OPERATION_UNWRAP_RESULT!( vulkan_mod::load_vulkan_lib() ); 
    INIT_OPERATION_UNWRAP_RESULT!( vulkan_mod::get_vulkan_available_extensions() );
    // vulkan_mod::list_available_extensions();
    let desired_extensions  = vec!["VK_KHR_display", "VK_KHR_surface"];
    INIT_OPERATION_UNWRAP_RESULT!( vulkan_mod::check_desired_extensions(&desired_extensions) );
    INIT_OPERATION_UNWRAP_RESULT!( vulkan_mod::instantiate_vulkan(&desired_extensions));
    println!("Done Loading");
}
