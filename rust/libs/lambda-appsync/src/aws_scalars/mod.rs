macro_rules! impl_new_string {
    ($name:ident) => {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
        #[serde(transparent)]
        pub struct $name(String);
        impl core::ops::Deref for $name {
            type Target = String;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                core::fmt::Display::fmt(&self.0, f)
            }
        }
        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }
        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

pub mod datetime;
pub mod email;
pub mod phone;
pub mod url;
