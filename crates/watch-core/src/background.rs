//! Build the custom clock background file for the watch.
//!
//! Format (reverse-engineered, see
//! dt-g1-research/docs/firmware-analysis/dial-background-format.md):
//! a 16-byte header followed by a 466x466 JPEG.
//!   0x00 u16 LE  = 0x0009 (image type)
//!   0x02 u24 LE  = JPEG length
//!   0x05 3 bytes = D2 21 1D (fixed signature)
//!   0x08 8 bytes = zero
//!   0x10 ...     = the JPEG
//! No checksum.

/// The square side the watch screen expects.
pub const SIDE: u32 = 466;
/// Path of the background file on the hidden partition.
pub const BG_PATH: &str = "res/user_clock_bg.bin";
/// Path of the thumbnail file on the hidden partition.
pub const THUMB_PATH: &str = "res/user_clock_thumb.bin";

const SIGNATURE: [u8; 3] = [0xD2, 0x21, 0x1D];

/// Wrap a JPEG (should be 466x466) in the 16-byte watch header.
pub fn wrap_jpeg(jpeg: &[u8]) -> Vec<u8> {
    let len = jpeg.len() as u32;
    let mut out = Vec::with_capacity(16 + jpeg.len());
    out.extend_from_slice(&[0x09, 0x00]);
    out.extend_from_slice(&len.to_le_bytes()[0..3]); // 24-bit little-endian length
    out.extend_from_slice(&SIGNATURE);
    out.extend_from_slice(&[0u8; 8]);
    out.extend_from_slice(jpeg);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_matches_real_sample() {
        // The real photo sample on the watch had a 116329-byte JPEG and header
        // 09 00 69 c6 01 d2 21 1d 00 00 00 00 00 00 00 00.
        let jpeg = vec![0u8; 116329];
        let out = wrap_jpeg(&jpeg);
        assert_eq!(
            &out[0..16],
            &[0x09, 0x00, 0x69, 0xc6, 0x01, 0xd2, 0x21, 0x1d, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(out.len(), 16 + 116329);
    }
}
