use std::{collections::HashMap, path::PathBuf};

fn main() {
    let library = HashMap::from([
        ("lucide".to_string(), PathBuf::from(lucide_slint::lib())),
        (
            // If you are not going to use Lucide Lab, remove this tuple
            "lucide-lab".to_string(),
            PathBuf::from(lucide_slint::lib_lab()),
        ),
    ]);
    let config = slint_build::CompilerConfiguration::new().with_library_paths(library);

    slint_build::compile_with_config("ui/app-window.slint", config).expect("Slint build failed");
}
