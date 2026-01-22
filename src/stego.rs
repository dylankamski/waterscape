//! Steganographic encoding using zero-width Unicode characters
//!
//! This module hides binary data within plain text using invisible Unicode characters.
//! The encoding is invisible to humans but can be detected and decoded by agents.
//!
//! ## Encoding scheme
//! - Zero-width space (U+200B) = 0
//! - Zero-width non-joiner (U+200C) = 1
//! - Zero-width joiner (U+200D) = separator between bytes
//!
//! Hidden data is inserted after each visible character in the cover text.

use crate::error::WaterscapeError;
use crate::Result;

const ZERO: char = '\u{200B}'; // Zero-width space = 0
const ONE: char = '\u{200C}';  // Zero-width non-joiner = 1
const SEP: char = '\u{200D}';  // Zero-width joiner = byte separator
const MARKER_START: &str = "\u{2060}"; // Word joiner - marks start of hidden data
const MARKER_END: &str = "\u{FEFF}";   // Zero-width no-break space - marks end

/// Encode binary data into zero-width characters
fn encode_byte(byte: u8) -> String {
    let mut result = String::with_capacity(9);
    for i in (0..8).rev() {
        if (byte >> i) & 1 == 1 {
            result.push(ONE);
        } else {
            result.push(ZERO);
        }
    }
    result.push(SEP);
    result
}

/// Decode zero-width characters back to a byte
fn decode_byte(chars: &[char]) -> Result<u8> {
    if chars.len() < 8 {
        return Err(WaterscapeError::Decoding("Not enough bits for a byte".into()));
    }

    let mut byte = 0u8;
    for (i, &c) in chars.iter().take(8).enumerate() {
        let bit = match c {
            ZERO => 0,
            ONE => 1,
            _ => return Err(WaterscapeError::Decoding(format!("Invalid bit character: {:?}", c))),
        };
        byte |= bit << (7 - i);
    }
    Ok(byte)
}

/// Encode binary data as zero-width string
pub fn encode_data(data: &[u8]) -> String {
    let mut result = String::with_capacity(data.len() * 9 + 2);
    result.push_str(MARKER_START);
    for &byte in data {
        result.push_str(&encode_byte(byte));
    }
    result.push_str(MARKER_END);
    result
}

/// Decode zero-width string back to binary data
pub fn decode_data(encoded: &str) -> Result<Vec<u8>> {
    let chars: Vec<char> = encoded.chars().collect();
    
    // Find start marker
    let start_pos = chars.iter().position(|&c| c == '\u{2060}')
        .ok_or(WaterscapeError::NoHiddenMessage)?;
    
    // Find end marker
    let end_pos = chars.iter().position(|&c| c == '\u{FEFF}')
        .ok_or(WaterscapeError::NoHiddenMessage)?;
    
    if end_pos <= start_pos {
        return Err(WaterscapeError::NoHiddenMessage);
    }
    
    // Extract only zero-width bit characters between markers
    let bit_chars: Vec<char> = chars[start_pos + 1..end_pos]
        .iter()
        .filter(|&&c| c == ZERO || c == ONE || c == SEP)
        .copied()
        .collect();
    
    // Split by separator and decode each byte
    let mut result = Vec::new();
    let mut current_bits = Vec::with_capacity(8);
    
    for c in bit_chars {
        if c == SEP {
            if current_bits.len() == 8 {
                result.push(decode_byte(&current_bits)?);
            }
            current_bits.clear();
        } else {
            current_bits.push(c);
        }
    }
    
    Ok(result)
}

/// Hide data within cover text by interleaving zero-width characters
pub fn hide_in_text(cover_text: &str, data: &[u8]) -> Result<String> {
    let encoded = encode_data(data);
    let cover_chars: Vec<char> = cover_text.chars().collect();
    let encoded_chars: Vec<char> = encoded.chars().collect();
    
    // We need at least some cover text
    if cover_chars.is_empty() {
        return Err(WaterscapeError::CoverTextTooShort {
            needed: 1,
            available: 0,
        });
    }
    
    // Distribute encoded characters throughout the cover text
    let mut result = String::with_capacity(cover_text.len() + encoded.len());
    let chunk_size = (encoded_chars.len() / cover_chars.len()).max(1);
    let mut encoded_idx = 0;
    
    for (i, cover_char) in cover_chars.iter().enumerate() {
        result.push(*cover_char);
        
        // Insert a chunk of encoded data after each cover character
        let end_idx = if i == cover_chars.len() - 1 {
            encoded_chars.len() // Put all remaining at the end
        } else {
            (encoded_idx + chunk_size).min(encoded_chars.len())
        };
        
        for &enc_char in &encoded_chars[encoded_idx..end_idx] {
            result.push(enc_char);
        }
        encoded_idx = end_idx;
    }
    
    // Append any remaining encoded characters
    for &enc_char in &encoded_chars[encoded_idx..] {
        result.push(enc_char);
    }
    
    Ok(result)
}

/// Extract hidden data from text with interleaved zero-width characters
pub fn extract_from_text(text: &str) -> Result<Vec<u8>> {
    // Extract all zero-width characters including markers
    let hidden: String = text
        .chars()
        .filter(|&c| c == ZERO || c == ONE || c == SEP || c == '\u{2060}' || c == '\u{FEFF}')
        .collect();
    
    if hidden.is_empty() {
        return Err(WaterscapeError::NoHiddenMessage);
    }
    
    decode_data(&hidden)
}

/// Extract visible text (remove all zero-width characters)
pub fn extract_visible_text(text: &str) -> String {
    text.chars()
        .filter(|&c| c != ZERO && c != ONE && c != SEP && c != '\u{2060}' && c != '\u{FEFF}')
        .collect()
}

/// Check if text contains hidden data
pub fn has_hidden_data(text: &str) -> bool {
    text.contains('\u{2060}') && text.contains('\u{FEFF}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_byte() {
        for byte in 0..=255u8 {
            let encoded = encode_byte(byte);
            let chars: Vec<char> = encoded.chars().collect();
            let decoded = decode_byte(&chars).unwrap();
            assert_eq!(byte, decoded);
        }
    }

    #[test]
    fn test_encode_decode_data() {
        let data = b"Hello, World!";
        let encoded = encode_data(data);
        let decoded = decode_data(&encoded).unwrap();
        assert_eq!(data.to_vec(), decoded);
    }

    #[test]
    fn test_hide_extract() {
        let cover = "This is a normal looking message.";
        let secret = b"Secret payload!";
        
        let hidden = hide_in_text(cover, secret).unwrap();
        
        // Visible text should be the same
        let visible = extract_visible_text(&hidden);
        assert_eq!(visible, cover);
        
        // Should be able to extract secret
        let extracted = extract_from_text(&hidden).unwrap();
        assert_eq!(extracted, secret.to_vec());
    }

    #[test]
    fn test_has_hidden_data() {
        let cover = "Normal text";
        let hidden = hide_in_text(cover, b"secret").unwrap();
        
        assert!(!has_hidden_data(cover));
        assert!(has_hidden_data(&hidden));
    }
}
