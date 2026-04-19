/// File-backed mmap region that exposes a [`SharedFlags`] to the Rust control
/// plane and, concurrently, to the Python DSPy pipeline process on the same
/// host.
///
/// # Lifecycle
///
/// ```text
/// Control plane (Rust)                Python pipeline
/// ─────────────────────────           ────────────────────────────
/// MmapRegion::open(path)              AgentFlags(path).open()
///   └─ creates/truncates file           └─ mmap.mmap(fd, SIZE)
///   └─ maps with memmap2                └─ reads/writes via struct
///   └─ resets flags to zero
///
/// flags().pipeline_active             mm[0:1] = b'\x01'
///   .store(true, Release)             mm.flush()
/// ```
///
/// # Safety
///
/// [`MmapRegion`] is `Send + Sync` because [`SharedFlags`] uses atomic types
/// for all mutable state.  The `session_id` byte array is written only from
/// the control plane (single writer) and read by Python (reader).
use std::path::Path;

use memmap2::MmapMut;
use tracing::info;

use super::flags::SharedFlags;

/// A memory-mapped region backed by a file, providing access to
/// [`SharedFlags`].
pub struct MmapRegion {
    /// Keeps the mapping alive.
    _mmap: MmapMut,
    /// Raw pointer into the mmap'd region, cast to `SharedFlags`.
    flags_ptr: *const SharedFlags,
}

// SAFETY: SharedFlags is Send+Sync (all fields are atomics or plain bytes).
// The raw pointer is stable for the lifetime of `_mmap` — mmap(2) returns a
// fixed virtual address that does not move when the MmapMut struct is moved.
unsafe impl Send for MmapRegion {}
unsafe impl Sync for MmapRegion {}

impl MmapRegion {
    /// Open (or create) the shared-memory file at `path` and map it.
    ///
    /// - Creates parent directories if they don't exist.
    /// - Truncates the file to `size_of::<SharedFlags>()` bytes.
    /// - Resets all flags to their zero/false state on every open so the
    ///   control plane always starts with a clean slate.
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        let target_len = std::mem::size_of::<SharedFlags>() as u64;
        file.set_len(target_len)?;

        // SAFETY: the file is open and has the correct length.
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };

        let flags_ptr = mmap.as_mut_ptr() as *const SharedFlags;

        // Zero the entire region so atomics start in a known state.
        // SAFETY: SharedFlags is repr(C); zero bytes are valid for all fields.
        unsafe {
            std::ptr::write_bytes(mmap.as_mut_ptr(), 0, mmap.len());
        }

        mmap.flush()?;

        info!(path = %path.display(), size = target_len, "mmap IPC region opened");

        Ok(Self { _mmap: mmap, flags_ptr })
    }

    /// Return a shared reference to the flag struct.
    ///
    /// The reference is valid for the lifetime of `self`.
    pub fn flags(&self) -> &SharedFlags {
        // SAFETY: `flags_ptr` was derived from the mmap buffer, which is alive
        // as long as `_mmap` lives (i.e., as long as `self` lives).
        unsafe { &*self.flags_ptr }
    }

    /// Flush dirty mmap pages to the underlying file.
    ///
    /// Call this after a batch of writes when you want the Python side to see
    /// the updated state immediately (the OS may otherwise defer writeback).
    pub fn flush(&self) -> anyhow::Result<()> {
        self._mmap.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use tempfile::tempdir;

    use super::*;

    fn make_region() -> (MmapRegion, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("agent-flags.shm");
        let region = MmapRegion::open(&path).unwrap();
        (region, dir)
    }

    #[test]
    fn open_creates_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("sub").join("agent-flags.shm");
        assert!(!path.exists());
        MmapRegion::open(&path).unwrap();
        assert!(path.exists());
        assert_eq!(path.metadata().unwrap().len(), 64);
    }

    #[test]
    fn flags_start_at_zero() {
        let (region, _dir) = make_region();
        let f = region.flags();
        assert!(!f.pipeline_active.load(Ordering::Acquire));
        assert!(!f.escalate_to_architect.load(Ordering::Acquire));
        assert!(!f.abort_requested.load(Ordering::Acquire));
        assert_eq!(f.load_confidence(), 0.0);
        assert_eq!(f.risk_level.load(Ordering::Acquire), 0);
        assert_eq!(f.load_session_id(), "");
    }

    #[test]
    fn pipeline_active_roundtrip() {
        let (region, _dir) = make_region();
        let f = region.flags();
        f.pipeline_active.store(true, Ordering::Release);
        assert!(f.pipeline_active.load(Ordering::Acquire));
    }

    #[test]
    fn confidence_and_session_persist_across_flush() {
        let (region, _dir) = make_region();
        let f = region.flags();
        f.store_confidence(0.88);
        f.store_session_id("test-session-uuid-1234567890123456");
        region.flush().unwrap();
        assert!((f.load_confidence() - 0.88).abs() < f32::EPSILON);
        assert!(f.load_session_id().starts_with("test-session-uuid"));
    }

    #[test]
    fn second_open_resets_flags() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("agent-flags.shm");

        {
            let r1 = MmapRegion::open(&path).unwrap();
            r1.flags().pipeline_active.store(true, Ordering::Release);
            r1.flags().store_confidence(0.5);
            r1.flush().unwrap();
        }

        // Second open resets to zero (control plane restart semantics).
        let r2 = MmapRegion::open(&path).unwrap();
        assert!(!r2.flags().pipeline_active.load(Ordering::Acquire));
        assert_eq!(r2.flags().load_confidence(), 0.0);
    }
}
