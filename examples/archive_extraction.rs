use std::io::Read;
use std::path::Path;

use librpm::archive::PackageReader;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("Usage: archive_extraction <path-to-rpm>");

    librpm::init().unwrap();

    let mut archive = PackageReader::open(Path::new(&path), None).unwrap();

    println!("Package: {}", archive.package().nevra());
    println!();

    let mut total_bytes: u64 = 0;
    let mut file_count: usize = 0;

    while let Some(mut entry) = archive.next_entry().unwrap() {
        let content_marker = if entry.has_content() { " " } else { "*" };
        println!(
            "{}{:>10}  {:o}  {}",
            content_marker,
            entry.size(),
            entry.mode(),
            entry.path(),
        );

        if entry.has_content() {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).unwrap();
            total_bytes += buf.len() as u64;
        }

        file_count += 1;
    }

    println!();
    println!("{file_count} files, {total_bytes} bytes of content extracted");
}
