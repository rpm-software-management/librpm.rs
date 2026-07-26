//! Tests for archive (payload) extraction.

use std::io::Read;
use std::path::Path;

use librpm::PackageHeader;
use librpm::archive::PackageReader;

mod common;

fn assets_path() -> std::path::PathBuf {
    common::get_assets_path().join("rpms")
}

fn basic_rpm() -> std::path::PathBuf {
    assets_path().join("rpm-basic-with-rsa4096-2.3.4-5.el9.noarch.rpm")
}

fn empty_rpm() -> std::path::PathBuf {
    assets_path().join("rpm-empty-0-0.x86_64.rpm")
}

#[test]
fn test_archive_entries() {
    common::configure();

    let pkg = PackageHeader::from_file(&basic_rpm()).unwrap();
    let non_ghost_files = pkg.files().iter().filter(|f| !f.flags().is_ghost()).count();

    let mut archive = PackageReader::open(&basic_rpm()).unwrap();
    let mut paths = Vec::new();

    while let Some(entry) = archive.next_entry().unwrap() {
        assert!(
            !entry.flags().is_ghost(),
            "ghost files should not appear in the archive: {}",
            entry.path(),
        );
        paths.push(entry.path());
    }

    assert_eq!(
        paths.len(),
        non_ghost_files,
        "archive entry count should match the number of non-ghost files"
    );
    assert!(paths.contains(&"/etc/rpm-basic/example_config.toml".to_string()));
    assert!(paths.contains(&"/usr/bin/rpm-basic".to_string()));
    assert!(paths.contains(&"/usr/share/doc/rpm-basic/README".to_string()));
}

#[test]
fn test_archive_read_content() {
    common::configure();

    let mut archive = PackageReader::open(&basic_rpm()).unwrap();

    while let Some(mut entry) = archive.next_entry().unwrap() {
        if entry.has_content() && entry.size() > 0 {
            let mut buf = Vec::new();
            let n = entry.read_to_end(&mut buf).unwrap();
            assert_eq!(
                n as u64,
                entry.size(),
                "read byte count should match size for {}",
                entry.path()
            );
            assert!(!buf.is_empty());
            return;
        }
    }
    panic!("no entry with readable content found");
}

#[test]
fn test_archive_read_specific_file() {
    common::configure();

    let mut archive = PackageReader::open(&basic_rpm()).unwrap();

    while let Some(mut entry) = archive.next_entry().unwrap() {
        if entry.path() == "/etc/rpm-basic/example_config.toml" {
            assert!(entry.has_content());
            assert_eq!(entry.mtime(), 1681068559);
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).unwrap();
            assert_eq!(buf.len(), 31);
            let content = String::from_utf8(buf).unwrap();
            assert!(!content.is_empty());
            return;
        }
    }
    panic!("config file not found in archive");
}

#[test]
fn test_archive_package_metadata() {
    common::configure();

    let archive = PackageReader::open(&basic_rpm()).unwrap();
    let pkg = archive.package();

    assert_eq!(pkg.name(), "rpm-basic");
    assert_eq!(pkg.version(), "2.3.4");
    assert_eq!(pkg.release(), "5.el9");
    assert_eq!(pkg.nevra(), "rpm-basic-1:2.3.4-5.el9.noarch");
}

#[test]
fn test_archive_empty_package() {
    common::configure();

    let mut archive = PackageReader::open(&empty_rpm()).unwrap();
    let mut count = 0;
    while let Some(_entry) = archive.next_entry().unwrap() {
        count += 1;
    }
    assert_eq!(count, 0, "empty package should have no archive entries");
}

#[test]
fn test_archive_exhausted_returns_none() {
    common::configure();

    let mut archive = PackageReader::open(&basic_rpm()).unwrap();

    while archive.next_entry().unwrap().is_some() {}

    assert!(
        archive.next_entry().unwrap().is_none(),
        "next_entry should keep returning None after exhaustion"
    );
}

#[test]
fn test_archive_nonexistent_file() {
    common::configure();

    let result = PackageReader::open(Path::new("/nonexistent/path/to/package.rpm"));
    assert!(result.is_err());
}
