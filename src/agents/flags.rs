/// Shared-memory flag layout for intra-host zero-copy IPC between the Rust
/// control plane and the Python DSPy pipeline.
///
/// # Memory layout (repr(C), align(64) — one cache line)
///
/// | Offset | Size | Field                  |
/// |--------|------|------------------------|
/// | 0      | 1    | pipeline_active        |
/// | 1      | 1    | escalate_to_architect  |
/// | 2      | 1    | abort_requested        |
/// | 3      | 1    | _pad0                  |
/// | 4      | 4    | junior_confidence (f32 bits) |
/// | 8      | 1    | risk_level             |
/// | 9      | 7    | _pad1                  |
/// | 16     | 36   | session_id ([u8; 36])  |
/// | 52     | 12   | _pad2                  |
/// | Total  | 64   |                        |
///
/// Python companion: `agents/neoland_agents/ipc/flags.py` uses the same offsets.
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};

/// Risk level values stored in `risk_level`.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

impl TryFrom<u8> for RiskLevel {
    type Error = u8;
    fn try_from(v: u8) -> Result<Self, u8> {
        match v {
            0 => Ok(Self::Low),
            1 => Ok(Self::Medium),
            2 => Ok(Self::High),
            3 => Ok(Self::Critical),
            other => Err(other),
        }
    }
}

/// Flat, cache-line-sized struct mapped directly onto the shared memory region.
///
/// All fields use atomic types so concurrent reads/writes from Rust and Python
/// (via OS-level mmap) are safe without an explicit mutex.
///
/// # Safety
///
/// This struct is `Send + Sync` because every field is an atomic type or a
/// plain byte array accessed only through the atomic wrapper methods provided
/// below.  Do **not** add non-atomic fields.
#[repr(C, align(64))]
pub struct SharedFlags {
    /// Set to `true` while the Python pipeline is processing a task.
    pub pipeline_active: AtomicBool,
    /// Set by Senior agent when the task should be escalated to Architect.
    pub escalate_to_architect: AtomicBool,
    /// Set by the control plane to request a graceful abort of the current run.
    pub abort_requested: AtomicBool,
    _pad0: u8,
    /// Junior agent confidence as `f32::to_bits()`.
    pub junior_confidence: AtomicU32,
    /// Current risk level (see [`RiskLevel`]).
    pub risk_level: AtomicU8,
    _pad1: [u8; 7],
    /// Active session UUID as a UTF-8 byte array, null-padded.
    pub session_id: [u8; 36],
    _pad2: [u8; 12],
}

// SAFETY: all mutable state goes through atomics; `session_id` is protected by
// the caller ensuring writes are coordinated (only one writer at a time).
unsafe impl Send for SharedFlags {}
unsafe impl Sync for SharedFlags {}

const _SIZE_CHECK: () = {
    // Compile-time assertion: must be exactly 64 bytes.
    // If this fails the Python layout is wrong too.
    assert!(std::mem::size_of::<SharedFlags>() == 64);
};

impl SharedFlags {
    /// Reset all flags to their zero/default state.
    pub fn reset(&self) {
        self.pipeline_active.store(false, Ordering::Release);
        self.escalate_to_architect.store(false, Ordering::Release);
        self.abort_requested.store(false, Ordering::Release);
        self.junior_confidence.store(0, Ordering::Release);
        self.risk_level.store(0, Ordering::Release);
        // session_id: zero via unsafe ptr (atomic store of [u8;36] not available)
        // SAFETY: `session_id` is plain bytes; no aliasing since we hold `&self`
        // exclusively during reset (called only from MmapRegion::open).
        unsafe {
            let ptr = self.session_id.as_ptr() as *mut u8;
            std::ptr::write_bytes(ptr, 0, 36);
        }
    }

    /// Store the active session UUID (up to 36 bytes, null-padded).
    pub fn store_session_id(&self, id: &str) {
        let bytes = id.as_bytes();
        let len = bytes.len().min(36);
        // SAFETY: plain byte write; caller must ensure no concurrent Python write.
        unsafe {
            let ptr = self.session_id.as_ptr() as *mut u8;
            std::ptr::write_bytes(ptr, 0, 36);
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
        }
    }

    /// Read the session UUID, stripping null bytes.
    pub fn load_session_id(&self) -> String {
        let end = self.session_id.iter().position(|&b| b == 0).unwrap_or(36);
        String::from_utf8_lossy(&self.session_id[..end]).into_owned()
    }

    /// Store junior confidence as f32.
    pub fn store_confidence(&self, v: f32) {
        self.junior_confidence.store(v.to_bits(), Ordering::Release);
    }

    /// Load junior confidence as f32.
    pub fn load_confidence(&self) -> f32 {
        f32::from_bits(self.junior_confidence.load(Ordering::Acquire))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn size_is_64_bytes() {
        assert_eq!(mem::size_of::<SharedFlags>(), 64);
    }

    #[test]
    fn align_is_64_bytes() {
        assert_eq!(mem::align_of::<SharedFlags>(), 64);
    }

    #[test]
    fn pipeline_active_offset_is_0() {
        let s = SharedFlags {
            pipeline_active: AtomicBool::new(false),
            escalate_to_architect: AtomicBool::new(false),
            abort_requested: AtomicBool::new(false),
            _pad0: 0,
            junior_confidence: AtomicU32::new(0),
            risk_level: AtomicU8::new(0),
            _pad1: [0; 7],
            session_id: [0; 36],
            _pad2: [0; 12],
        };
        let base = &s as *const SharedFlags as usize;
        let field = &s.pipeline_active as *const AtomicBool as usize;
        assert_eq!(field - base, 0);
    }

    #[test]
    fn junior_confidence_offset_is_4() {
        let s = SharedFlags {
            pipeline_active: AtomicBool::new(false),
            escalate_to_architect: AtomicBool::new(false),
            abort_requested: AtomicBool::new(false),
            _pad0: 0,
            junior_confidence: AtomicU32::new(0),
            risk_level: AtomicU8::new(0),
            _pad1: [0; 7],
            session_id: [0; 36],
            _pad2: [0; 12],
        };
        let base = &s as *const SharedFlags as usize;
        let field = &s.junior_confidence as *const AtomicU32 as usize;
        assert_eq!(field - base, 4);
    }

    #[test]
    fn session_id_offset_is_16() {
        let s = SharedFlags {
            pipeline_active: AtomicBool::new(false),
            escalate_to_architect: AtomicBool::new(false),
            abort_requested: AtomicBool::new(false),
            _pad0: 0,
            junior_confidence: AtomicU32::new(0),
            risk_level: AtomicU8::new(0),
            _pad1: [0; 7],
            session_id: [0; 36],
            _pad2: [0; 12],
        };
        let base = &s as *const SharedFlags as usize;
        let field = s.session_id.as_ptr() as usize;
        assert_eq!(field - base, 16);
    }

    #[test]
    fn store_and_load_confidence() {
        let s = SharedFlags {
            pipeline_active: AtomicBool::new(false),
            escalate_to_architect: AtomicBool::new(false),
            abort_requested: AtomicBool::new(false),
            _pad0: 0,
            junior_confidence: AtomicU32::new(0),
            risk_level: AtomicU8::new(0),
            _pad1: [0; 7],
            session_id: [0; 36],
            _pad2: [0; 12],
        };
        s.store_confidence(0.75);
        let loaded = s.load_confidence();
        assert!((loaded - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn store_and_load_session_id() {
        let s = SharedFlags {
            pipeline_active: AtomicBool::new(false),
            escalate_to_architect: AtomicBool::new(false),
            abort_requested: AtomicBool::new(false),
            _pad0: 0,
            junior_confidence: AtomicU32::new(0),
            risk_level: AtomicU8::new(0),
            _pad1: [0; 7],
            session_id: [0; 36],
            _pad2: [0; 12],
        };
        let id = "550e8400-e29b-41d4-a716-446655440000";
        s.store_session_id(id);
        assert_eq!(s.load_session_id(), id);
    }

    #[test]
    fn risk_level_roundtrip() {
        assert_eq!(RiskLevel::try_from(0), Ok(RiskLevel::Low));
        assert_eq!(RiskLevel::try_from(3), Ok(RiskLevel::Critical));
        assert_eq!(RiskLevel::try_from(4), Err(4u8));
    }
}
