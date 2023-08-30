use std::path::Path;

pub fn is_wasm_file(path_str: &str) -> bool {
    let path = Path::new(path_str);

    if !path.is_dir() {
        if let Some(extension) = path.extension() {
            if let Some(extension_str) = extension.to_str() {
                return extension_str == "wasm";
            }
        }
    }

    false
}