//! Manage xml character escapes

use std::borrow::Cow;
use error::{Error, ResultPos};
use AsStr;
use std::char;

/// helper function to unescape a `&[u8]` and replace all
/// xml escaped characters ('&...;') into their corresponding value
pub fn unescape(raw: &[u8]) -> ResultPos<Cow<[u8]>> {
    let mut escapes = Vec::new();
    let mut bytes = raw.iter().enumerate();
    while let Some((i, &b)) = bytes.next() {
        if b == b'&' {
            if let Some((j, _)) = bytes.find(|&(_, &b)| b == b';') {
                // search for character correctness
                // copied and modified from xml-rs inside_reference.rs
                match &raw[(i + 1)..j] {
                    b"lt" => escapes.push((i..j, b'<')),
                    b"gt" => escapes.push((i..j, b'>')),
                    b"amp" => escapes.push((i..j, b'&')),
                    b"apos" => escapes.push((i..j, b'\'')),
                    b"quot" => escapes.push((i..j, b'\"')),
                    b"" => return Err((Error::Malformed(
                                "Encountered empty entity".to_owned()), i)),
                    b"#x0" | b"#0" => return Err((Error::Malformed(
                                "Null character entity is not allowed".to_owned()), i)),
                    bytes if bytes.len() > 1 && bytes[0] == b'#' => {
                        if bytes[1] == b'x' {
                            let name = try!(bytes[2..].as_str().map_err(|e| (Error::from(e), i))); 
                            match u32::from_str_radix(name, 16).ok().and_then(char::from_u32) {
                                Some(c) => escapes.push((i..j, c as u8)),
                                None    => return Err((Error::Malformed(format!(
                                    "Invalid hexadecimal character number in an entity: {}", name)), i)),
                            }
                        } else {
                            let name = try!(bytes[1..].as_str().map_err(|e| (Error::from(e), i))); 
                            match u32::from_str_radix(name, 8).ok().and_then(char::from_u32) {
                                Some(c) => escapes.push((i..j, c as u8)),
                                None    => return Err((Error::Malformed(format!(
                                    "Invalid decimal character number in an entity: {}", name)), i))
                            }
                        }
                    },
                    bytes => return Err((Error::Malformed(format!(
                                    "Unexpected entity: {:?}", bytes.as_str())), i)),
                }
            } else {
                return Err((Error::Malformed("Cannot find ';' after '&'".to_owned()), i));
            }
        }
    }
    if escapes.is_empty() {
        Ok(Cow::Borrowed(raw))
    } else {
        let len = escapes.iter().fold(raw.len(), |c, &(ref r, _)| c - (r.end - r.start));
        let mut v = Vec::with_capacity(len);
        let mut start = 0;
        for (r, b) in escapes {
            v.extend_from_slice(&raw[start..r.start]);
            v.push(b);
            start = r.end + 1;
        }
        if start < len {
            v.extend_from_slice(&raw[start..]);
        }
        Ok(Cow::Owned(v))
    }
}
