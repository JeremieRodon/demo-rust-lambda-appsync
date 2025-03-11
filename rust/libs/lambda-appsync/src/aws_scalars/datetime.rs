use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AWSTimestamp(u64);
impl AWSTimestamp {
    pub fn as_u64(self) -> u64 {
        self.0
    }
}
impl From<u64> for AWSTimestamp {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
impl From<AWSTimestamp> for u64 {
    fn from(value: AWSTimestamp) -> Self {
        let t = AWSDateTime("blabla".to_owned());
        println!("{t}");
        value.as_u64()
    }
}

impl_new_string!(AWSDateTime);
impl_new_string!(AWSDate);
impl_new_string!(AWSTime);
