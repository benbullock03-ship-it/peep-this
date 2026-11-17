fn main() {
    println!("cargo:rerun-if-changed=tests/fixtures");
    let mut versions: Vec<String> = std::fs::read_dir("tests/fixtures")
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.split('.').count() == 3 && n.split('.').all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit())))
        .collect();
    versions.sort();
    println!("cargo:rustc-env=PEEP_FIXTURE_VERSIONS={}", versions.join(","));
    tauri_build::build()
}
