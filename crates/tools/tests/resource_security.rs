use devfoundry_tools::archive::{
    ArchiveEntry, ArchiveEntryKind, ArchiveLimits, sha256_hex, validate_entries,
};

fn entry(path: &str, size: u64) -> ArchiveEntry {
    ArchiveEntry {
        path: path.into(),
        size,
        kind: ArchiveEntryKind::File,
        content: vec![0; size as usize],
    }
}

#[test]
fn rejects_archive_traversal_before_staging() {
    let error =
        validate_entries(&[entry("../outside.txt", 1)], &ArchiveLimits::default()).unwrap_err();
    assert!(error.to_string().contains("traversal"));
}

#[test]
fn rejects_symlinks_and_duplicate_normalized_targets() {
    let symlink = ArchiveEntry {
        path: "link".into(),
        size: 0,
        kind: ArchiveEntryKind::Symlink,
        content: Vec::new(),
    };
    assert!(validate_entries(&[symlink], &ArchiveLimits::default()).is_err());

    let duplicate = [entry("a/../file", 1), entry("file", 1)];
    assert!(validate_entries(&duplicate, &ArchiveLimits::default()).is_err());
}

#[test]
fn rejects_archive_bomb_limits_without_writing_files() {
    let limits = ArchiveLimits {
        max_files: 1,
        max_expanded_bytes: 2,
        max_file_bytes: 2,
        max_path_bytes: 32,
    };
    let error = validate_entries(&[entry("a", 2), entry("b", 1)], &limits).unwrap_err();
    assert!(error.to_string().contains("file count"));
}

#[test]
fn hashes_content_with_sha256() {
    assert_eq!(sha256_hex(b"resource").len(), 64);
}
