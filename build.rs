// build.rs

use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("tests.rs");
    let mut test_file = fs::File::create(dest_path).unwrap();

    for entry in fs::read_dir("./tests/snapshots").unwrap() {
        let snapshot_path = entry.unwrap().path();
        let file_name = snapshot_path.file_name();
        if !snapshot_path.is_dir() {
            continue;
        }
        if file_name.is_none() {
            continue;
        }
        let snapshot_name = file_name.unwrap().to_str().unwrap();
        write!(
            test_file,
            "
            #[test]
            pub fn test_snapshot_{snapshot_name}() {{
                test_snapshot(\"{}\");
            }}
            ",
            snapshot_path.to_str().unwrap()
        )
        .unwrap();
    }
    println!("cargo:rerun-if-changed=tests/snapshots");
}
