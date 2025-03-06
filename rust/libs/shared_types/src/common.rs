mod new_uuid {
    use std::ops::Deref;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(try_from = "String", into = "String")]
    pub struct Uuid(uuid::Uuid);
    impl Uuid {
        pub fn new_v4() -> Self {
            Self(uuid::Uuid::new_v4())
        }
    }
    impl TryFrom<String> for Uuid {
        type Error = uuid::Error;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            Ok(Uuid(uuid::Uuid::parse_str(&value)?))
        }
    }
    impl core::fmt::Display for Uuid {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl From<Uuid> for String {
        fn from(value: Uuid) -> Self {
            value.to_string()
        }
    }
    impl Deref for Uuid {
        type Target = uuid::Uuid;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}
pub use new_uuid::Uuid;
