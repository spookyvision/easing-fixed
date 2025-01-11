// clean up test results dir
// sadly we can't #[cfg(test)] this, so it runs on every build. Oh well.
use std::{fs::remove_file, path::PathBuf};

use walkdir::WalkDir;
pub fn main() {
    let root = option_env!("CARGO_MANIFEST_DIR").unwrap();
    let mut root = PathBuf::from(root);
    root.push("jupyter-tests");

    // find all .json files directly in jupyter-tests/
    for entry in WalkDir::new(root).max_depth(1) {
        if let Ok(entry) = entry {
            let path = entry.path();

            // convert to string because *path* ends_with() matching only considers whole path components
            let path_s = path.to_string_lossy();
            if path_s.ends_with(".json") {
                if let Err(e) = remove_file(&path) {
                    println!("cargo:warning=could not delete {}: {:?}", path.display(), e);
                }
            }
        }
    }
}
