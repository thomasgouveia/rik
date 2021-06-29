use std::path::PathBuf;

pub fn find_binary(binary: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).filter_map(|dir| {
            let full_path = dir.join(binary);
            if full_path.is_file() {
                Some(full_path)
            } else {
                None
            }
        })
            .next()
    })
}