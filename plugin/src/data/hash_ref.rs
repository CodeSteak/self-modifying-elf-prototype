use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct HashRef(#[serde(with = "hash_ref_fmt")] pub [u8; 32]);

mod hash_ref_fmt {
    use crate::data::hash_ref::HashRef;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(data: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", &HashRef(*data));
        serializer.serialize_str(&s)
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::str::FromStr;
        let s = String::deserialize(deserializer)?;
        HashRef::from_str(&s)
            .map(|hr| hr.0)
            .map_err(|_e| serde::de::Error::custom("Invalid HashRef"))
    }
}

impl HashRef {
    pub fn from_data(data: &[u8]) -> Self {
        let res = blake2_rfc::blake2b::blake2b(32, &[], &data);

        Self::from_result(res)
    }

    pub fn from_result(res: blake2_rfc::blake2b::Blake2bResult) -> Self {
        let mut ret = HashRef([0u8; 32]);

        if res.len() != 32 {
            panic!("HashRef expects 32Byte == 256Bit Hash.");
        }

        for (pos, byte) in res.as_bytes().iter().enumerate() {
            ret.0[pos] = *byte;
        }

        ret
    }
}

use std::fmt;
impl fmt::Debug for HashRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for HashRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in self.0.iter() {
            write!(f, "{:02X}", i)?;
        }
        Ok(())
    }
}

use std::str::FromStr;
impl FromStr for HashRef {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.bytes().len() != 64 {
            return Err(());
        }

        fn hex2num(n: u8) -> Option<u8> {
            match n {
                48...58 => Some(n - '0' as u8),
                97...103 => Some(n + 10 - 'a' as u8),
                65...71 => Some(n + 10 - 'A' as u8),
                _ => None,
            }
        }

        let mut result = Self([0u8; 32]);
        let mut bytes = s.bytes();

        for b in result.0.iter_mut() {
            let high = bytes.next().and_then(hex2num).ok_or(())?;
            let low = bytes.next().and_then(hex2num).ok_or(())?;

            *b = (high << 4) | low;
        }

        Ok(result)
    }
}

use std::hash::{Hash, Hasher};
impl Hash for HashRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for i in self.0.iter() {
            state.write_u8(*i);
        }
    }
}

impl Eq for HashRef {}
impl PartialEq for HashRef {
    fn eq(&self, other: &HashRef) -> bool {
        for (a, b) in self.0.iter().zip(other.0.iter()) {
            if a != b {
                return false;
            }
        }

        return true;
    }
}

use std::cmp::Ordering;
impl PartialOrd for HashRef {
    fn partial_cmp(&self, other: &HashRef) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for HashRef {
    fn cmp(&self, other: &HashRef) -> Ordering {
        self.0.cmp(&other.0)
    }
}
