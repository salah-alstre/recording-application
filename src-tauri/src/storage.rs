//! Disk-space queries and the library size-limit cleanup planner.
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskSpace {
    pub free_bytes: u64,
    pub total_bytes: u64,
}

/// Free / total bytes on the volume containing `path` (walks up to the nearest existing ancestor).
pub fn disk_space(path: &Path) -> Option<DiskSpace> {
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    let mut p = path;
    while !p.exists() {
        p = p.parent()?;
    }
    let wide: Vec<u16> = p
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let (mut free, mut total, mut total_free) = (0u64, 0u64, 0u64);
    unsafe {
        GetDiskFreeSpaceExW(
            PCWSTR(wide.as_ptr()),
            Some(&mut free),
            Some(&mut total),
            Some(&mut total_free),
        )
        .ok()?;
    }
    Some(DiskSpace {
        free_bytes: free,
        total_bytes: total,
    })
}

/// Total size of regular files below `dir`.
pub fn dir_size(dir: &Path) -> u64 {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return 0;
    };
    rd.flatten()
        .map(|e| match e.metadata() {
            Ok(m) if m.is_dir() => dir_size(&e.path()),
            Ok(m) => m.len(),
            Err(_) => 0,
        })
        .sum()
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: String,
    pub size: u64,
    pub created_at: i64,
    pub favorite: bool,
}

/// Which entries to delete (oldest first) so that the library fits into `limit_bytes`.
/// Favorites are never selected, even if that means the limit cannot be reached.
pub fn plan_cleanup(entries: &[Entry], limit_bytes: u64) -> Vec<String> {
    let mut total: u64 = entries.iter().map(|e| e.size).sum();
    if limit_bytes == 0 || total <= limit_bytes {
        return vec![];
    }
    let mut candidates: Vec<&Entry> = entries.iter().filter(|e| !e.favorite).collect();
    candidates.sort_by_key(|e| e.created_at);
    let mut out = vec![];
    for e in candidates {
        if total <= limit_bytes {
            break;
        }
        total = total.saturating_sub(e.size);
        out.push(e.id.clone());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(id: &str, size: u64, t: i64, fav: bool) -> Entry {
        Entry {
            id: id.into(),
            size,
            created_at: t,
            favorite: fav,
        }
    }

    #[test]
    fn deletes_oldest_first_until_under_limit() {
        let entries = [
            e("a", 100, 1, false),
            e("b", 100, 2, false),
            e("c", 100, 3, false),
        ];
        assert_eq!(plan_cleanup(&entries, 200), vec!["a"]);
        assert_eq!(plan_cleanup(&entries, 100), vec!["a", "b"]);
    }

    #[test]
    fn favorites_are_never_deleted() {
        let entries = [e("fav", 500, 1, true), e("b", 100, 2, false)];
        assert_eq!(plan_cleanup(&entries, 10), vec!["b"]);
    }

    #[test]
    fn no_limit_or_under_limit_deletes_nothing() {
        let entries = [e("a", 100, 1, false)];
        assert!(plan_cleanup(&entries, 0).is_empty());
        assert!(plan_cleanup(&entries, 1000).is_empty());
    }

    #[test]
    fn disk_space_reports_something_for_temp() {
        let d = disk_space(&std::env::temp_dir()).expect("disk space");
        assert!(d.total_bytes > 0 && d.free_bytes <= d.total_bytes);
    }

    #[test]
    fn dir_size_sums_nested_files() {
        let dir = std::env::temp_dir().join(format!("rimlight-size-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("a"), vec![0u8; 10]).unwrap();
        std::fs::write(dir.join("sub").join("b"), vec![0u8; 5]).unwrap();
        assert_eq!(dir_size(&dir), 15);
        let _ = std::fs::remove_dir_all(dir);
    }
}
