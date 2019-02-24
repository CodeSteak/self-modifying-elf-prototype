pub use ipc::*;
pub use plugin::*;

pub use actix_web::dev::Resource;
pub use actix_web::*;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) ctx: Channel,
}

pub(crate) fn url_encode(s: &str) -> String {
    let mut result = String::new();

    for b in s.as_bytes().iter() {
        match b {
            48...58 => result.push(*b as char),
            65...91 => result.push(*b as char),
            97...123 => result.push(*b as char),
            _ => {
                result += &format!("%{:02X}", b);
            }
        }
    }

    result
}

pub(crate) fn html_encode(s: &str) -> String {
    let mut result = String::new();

    for b in s.chars() {
        match b {
            ' ' => result.push(b),
            '\t' => result.push(b),
            '0'..='9' => result.push(b),
            'a'..='z' => result.push(b),
            'A'..='Z' => result.push(b),
            '\n' => {
                result += &format!("<br/>");
            }
            _ => {
                result += &format!("&#x{:04X};", b as u32);
            }
        }
    }

    result
}

pub(crate) fn url_decode(s: &str) -> Option<String> {
    let mut data: Vec<u8> = vec![];
    let mut bytes = s.as_bytes().iter();

    fn hex2num(n: &u8) -> Option<u8> {
        match *n {
            48...58 => Some(*n - '0' as u8),
            97...103 => Some(*n + 10 - 'a' as u8),
            65...71 => Some(*n + 10 - 'A' as u8),
            _ => None,
        }
    }

    while let Some(b) = bytes.next() {
        if *b == ('%' as u8) {
            let high = bytes.next().and_then(hex2num)?;
            let low = bytes.next().and_then(hex2num)?;
            data.push((high << 4) | low);
        } else {
            data.push(*b);
        }
    }

    String::from_utf8(data).ok()
}
