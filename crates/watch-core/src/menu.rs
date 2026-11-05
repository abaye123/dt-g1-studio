//! Parsing and editing of the watch menu file `menu_list.cfg`.
//!
//! Format (reverse-engineered, see dt-g1-research/docs/firmware-analysis):
//! - Exactly 1600 bytes.
//! - 100 slots of 16 bytes each, little-endian.
//! - Each slot: AppID (u32), IconID (u32), NameID (u32), Flags (u32).
//! - Flags: 1 = visible, 0 = hidden/ignored. An empty slot has AppID == 0.
//!
//! A corrupt file can boot-loop the watch, so this editor only ever changes
//! known fields and always keeps the byte length at exactly 1600.

use crate::Error;

pub const MENU_SIZE: usize = 1600;
pub const SLOT_SIZE: usize = 16;
pub const SLOT_COUNT: usize = MENU_SIZE / SLOT_SIZE; // 100

/// One 16-byte menu slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    pub app_id: u32,
    pub icon_id: u32,
    pub name_id: u32,
    pub flags: u32,
}

impl Slot {
    pub const EMPTY: Slot = Slot { app_id: 0, icon_id: 0, name_id: 0, flags: 0 };

    pub fn is_empty(&self) -> bool {
        self.app_id == 0
    }

    pub fn is_visible(&self) -> bool {
        self.app_id != 0 && self.flags == 1
    }
}

/// The full 100-slot menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Menu {
    pub slots: [Slot; SLOT_COUNT],
}

fn rd_u32(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

impl Menu {
    /// Parse a raw `menu_list.cfg`. Fails if the length is not exactly 1600.
    pub fn parse(bytes: &[u8]) -> Result<Menu, Error> {
        if bytes.len() != MENU_SIZE {
            return Err(Error::MenuSize(bytes.len()));
        }
        let mut slots = [Slot::EMPTY; SLOT_COUNT];
        for (i, slot) in slots.iter_mut().enumerate() {
            let o = i * SLOT_SIZE;
            *slot = Slot {
                app_id: rd_u32(bytes, o),
                icon_id: rd_u32(bytes, o + 4),
                name_id: rd_u32(bytes, o + 8),
                flags: rd_u32(bytes, o + 12),
            };
        }
        Ok(Menu { slots })
    }

    /// Serialize back to exactly 1600 bytes.
    pub fn to_bytes(&self) -> [u8; MENU_SIZE] {
        let mut out = [0u8; MENU_SIZE];
        for (i, slot) in self.slots.iter().enumerate() {
            let o = i * SLOT_SIZE;
            out[o..o + 4].copy_from_slice(&slot.app_id.to_le_bytes());
            out[o + 4..o + 8].copy_from_slice(&slot.icon_id.to_le_bytes());
            out[o + 8..o + 12].copy_from_slice(&slot.name_id.to_le_bytes());
            out[o + 12..o + 16].copy_from_slice(&slot.flags.to_le_bytes());
        }
        out
    }

    /// Indices of the non-empty slots, in order.
    pub fn used_indices(&self) -> Vec<usize> {
        (0..SLOT_COUNT).filter(|&i| !self.slots[i].is_empty()).collect()
    }

    /// Set the visible flag of the first slot with this AppID. Returns true if found.
    pub fn set_visible(&mut self, app_id: u32, visible: bool) -> bool {
        for slot in self.slots.iter_mut() {
            if slot.app_id == app_id {
                slot.flags = if visible { 1 } else { 0 };
                return true;
            }
        }
        false
    }

    /// Hide several AppIDs at once. Returns the AppIDs that were not found.
    pub fn hide_all(&mut self, app_ids: &[u32]) -> Vec<u32> {
        app_ids
            .iter()
            .copied()
            .filter(|&id| !self.set_visible(id, false))
            .collect()
    }

    /// Move the slot at `from` to `to`, shifting the slots in between.
    /// Indices are positions in the full 100-slot array.
    pub fn move_slot(&mut self, from: usize, to: usize) -> Result<(), Error> {
        if from >= SLOT_COUNT || to >= SLOT_COUNT {
            return Err(Error::MenuIndex(from.max(to)));
        }
        if from == to {
            return Ok(());
        }
        let moved = self.slots[from];
        if from < to {
            for i in from..to {
                self.slots[i] = self.slots[i + 1];
            }
        } else {
            for i in (to + 1..=from).rev() {
                self.slots[i] = self.slots[i - 1];
            }
        }
        self.slots[to] = moved;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> [u8; MENU_SIZE] {
        let mut b = [0u8; MENU_SIZE];
        // slot 0: AppID 0xC461, icon 1, name 2, visible
        b[0..4].copy_from_slice(&0xC461u32.to_le_bytes());
        b[4..8].copy_from_slice(&1u32.to_le_bytes());
        b[8..12].copy_from_slice(&2u32.to_le_bytes());
        b[12..16].copy_from_slice(&1u32.to_le_bytes());
        // slot 1: AppID 0xC759 (video), visible
        b[16..20].copy_from_slice(&0xC759u32.to_le_bytes());
        b[28..32].copy_from_slice(&1u32.to_le_bytes());
        b
    }

    #[test]
    fn round_trip_is_identity() {
        let b = sample();
        let m = Menu::parse(&b).unwrap();
        assert_eq!(m.to_bytes(), b);
    }

    #[test]
    fn wrong_size_is_rejected() {
        assert!(Menu::parse(&[0u8; 1599]).is_err());
        assert!(Menu::parse(&[0u8; 1601]).is_err());
    }

    #[test]
    fn hiding_sets_flag_and_keeps_size() {
        let mut m = Menu::parse(&sample()).unwrap();
        assert!(m.slots[1].is_visible());
        let missing = m.hide_all(&[0xC759]);
        assert!(missing.is_empty());
        assert!(!m.slots[1].is_visible());
        assert_eq!(m.to_bytes().len(), MENU_SIZE);
    }

    #[test]
    fn hiding_unknown_id_is_reported() {
        let mut m = Menu::parse(&sample()).unwrap();
        let missing = m.hide_all(&[0xDEAD]);
        assert_eq!(missing, vec![0xDEAD]);
    }

    #[test]
    fn move_slot_reorders() {
        let mut m = Menu::parse(&sample()).unwrap();
        let a = m.slots[0];
        let b = m.slots[1];
        m.move_slot(0, 1).unwrap();
        assert_eq!(m.slots[0], b);
        assert_eq!(m.slots[1], a);
    }
}
