//! Core identifiers and shared lightweight types for UltraSearch.
//!
//! These types intentionally avoid heavy dependencies and aim to be
//! serialization-friendly for rkyv/bincode and IPC payloads.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub type VolumeId = u16;
pub type FileId = u64;
pub type Timestamp = i64; // Unix timestamp (seconds); i64 for easy serde and fast fields.

/// Packed identifier combining a volume id and NTFS file reference number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DocKey(pub u64);

impl DocKey {
    /// Pack a `VolumeId` (high bits) and `FileId` (low bits) into a `DocKey`.
    pub const fn from_parts(volume: VolumeId, file: FileId) -> Self {
        // Use the upper 16 bits for the volume and the remaining 48 bits for the FRN.
        let packed = ((volume as u64) << 48) | (file & 0x0000_FFFF_FFFF_FFFF);
        DocKey(packed)
    }

    /// Split the packed id back into `(VolumeId, FileId)`.
    pub const fn into_parts(self) -> (VolumeId, FileId) {
        let volume = (self.0 >> 48) as VolumeId;
        let file = self.0 & 0x0000_FFFF_FFFF_FFFF;
        (volume, file)
    }

    /// Return the volume id component.
    pub const fn volume(self) -> VolumeId {
        (self.0 >> 48) as VolumeId
    }

    /// Return the file id (FRN) component.
    pub const fn file_id(self) -> FileId {
        self.0 & 0x0000_FFFF_FFFF_FFFF
    }
}

impl core::fmt::Display for DocKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (v, id) = self.into_parts();
        write!(f, "{}:{:#013x}", v, id)
    }
}

impl FromStr for DocKey {
    type Err = &'static str;

    /// Parses the Display form: `<volume>:0x<frn_hex>`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (vol_part, frn_part) = s.split_once(':').ok_or("missing ':'")?;
        let volume: VolumeId = vol_part.parse().map_err(|_| "invalid volume id")?;
        let frn_hex = frn_part.strip_prefix("0x").ok_or("missing 0x prefix")?;
        let file = u64::from_str_radix(frn_hex, 16).map_err(|_| "invalid frn hex")?
            & 0x0000_FFFF_FFFF_FFFF;
        Ok(DocKey::from_parts(volume, file))
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct FileFlags: u32 {
        const IS_DIR   = 0b0000_0001;
        const HIDDEN   = 0b0000_0010;
        const SYSTEM   = 0b0000_0100;
        const ARCHIVE  = 0b0000_1000;
        const REPARSE  = 0b0001_0000;
        const OFFLINE  = 0b0010_0000;
        const TEMPORARY= 0b0100_0000;
    }
}

/// Minimal metadata carried through indexing pipelines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMeta {
    pub key: DocKey,
    pub volume: VolumeId,
    pub parent: Option<DocKey>,
    pub name: String,
    pub ext: Option<String>,
    pub path: Option<String>,
    pub size: u64,
    pub created: Timestamp,
    pub modified: Timestamp,
    pub flags: FileFlags,
}

impl FileMeta {
    /// Create a new FileMeta, deriving extension if not provided.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key: DocKey,
        volume: VolumeId,
        parent: Option<DocKey>,
        name: String,
        path: Option<String>,
        size: u64,
        created: Timestamp,
        modified: Timestamp,
        flags: FileFlags,
    ) -> Self {
        let ext = name
            .rsplit_once('.')
            .map(|(_, ext)| ext.to_ascii_lowercase());
        Self {
            key,
            volume,
            parent,
            name,
            ext,
            path,
            size,
            created,
            modified,
            flags,
        }
    }
}

/// Per-volume configuration snapshot (kept simple for now).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSettings {
    pub volume: VolumeId,
    pub include_paths: Vec<String>,
    pub exclude_paths: Vec<String>,
    pub content_indexing: bool,
}

/// Basic descriptor for a discovered NTFS volume.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeDescriptor {
    pub id: VolumeId,
    /// NT-style volume GUID path, e.g. `\\\\?\\Volume{...}\\`
    pub guid_path: String,
    /// Optional drive letters mapped to this volume, e.g. ["C:", "D:"].
    pub drive_letters: Vec<String>,
}

pub mod config;

impl FileFlags {
    pub fn is_dir(self) -> bool {
        self.contains(Self::IS_DIR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_key_round_trips() {
        let dk = DocKey::from_parts(42, 0x1234_5678_9abc);
        let (v, f) = dk.into_parts();
        assert_eq!(v, 42);
        assert_eq!(f, 0x1234_5678_9abc);
    }

    #[test]
    fn file_meta_ext_derives_lowercase() {
        let key = DocKey::from_parts(1, 2);
        let fm = FileMeta::new(
            key,
            1,
            None,
            "Report.PDF".to_string(),
            None,
            10,
            0,
            0,
            FileFlags::empty(),
        );
        assert_eq!(fm.ext.as_deref(), Some("pdf"));
    }

    #[test]
    fn doc_key_display_is_stable() {
        let dk = DocKey::from_parts(7, 0xabc);
        assert_eq!(dk.to_string(), "7:0x000000000abc");
    }

    #[test]
    fn doc_key_parse_round_trip() {
        let original = DocKey::from_parts(9, 0xfeed_beef);
        let parsed: DocKey = original.to_string().parse().unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn volume_descriptor_holds_letters() {
        let vd = VolumeDescriptor {
            id: 1,
            guid_path: r"\\?\Volume{abc}\\".to_string(),
            drive_letters: vec!["C:".into(), "D:".into()],
        };
        assert_eq!(vd.id, 1);
        assert_eq!(vd.drive_letters.len(), 2);
    }
}
