/// Create a transparent wrapper with Deref, DerefMut, From, and sqlx_json_decode
/// Usage: make_transparent!(#[derive(Debug, Clone)] pub struct MyType(InnerType));
/// Usage with generics: make_transparent!(#[derive(Debug, Clone)] pub struct MyType<T>(Option<T>));
/// Requires: use std::ops::{Deref, DerefMut}; and sqlx_transparent_json_decode::sqlx_json_decode; in the calling module
macro_rules! make_transparent {
    // Handle generic types like JsonOption<T>
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident<$generic:ident>($inner:ty)
    ) => {
        $(#[$attr])*
        $vis struct $name<$generic>($inner);

        impl<$generic> std::ops::Deref for $name<$generic> {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<$generic> std::ops::DerefMut for $name<$generic> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl<$generic> From<$inner> for $name<$generic> {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }


        // Implement Default to create empty JsonOption
        impl<$generic> Default for $name<$generic> {
            fn default() -> Self {
                Self(None)
            }
        }

        // Add a custom method for JSON conversion that doesn't conflict with From
        impl<$generic> $name<$generic>
        where
            $generic: serde::de::DeserializeOwned,
        {
            pub fn from_json_option(value: Option<serde_json::Value>) -> Self {
                match value {
                    Some(json_value) => {
                        match serde_json::from_value::<$generic>(json_value) {
                            Ok(parsed) => $name(Some(parsed)),
                            Err(_) => $name(None),
                        }
                    }
                    None => $name(None),
                }
            }
        }

        // Instead of fighting the orphan rule, let's implement the SQLx traits directly
        // This makes JsonOption work seamlessly with SQLx without needing From implementations
        impl<$generic> sqlx::Decode<'_, sqlx::Postgres> for $name<$generic>
        where
            $generic: serde::de::DeserializeOwned,
        {
            fn decode(
                value: sqlx::postgres::PgValueRef<'_>,
            ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
                let json_value = sqlx::types::Json::<serde_json::Value>::decode(value)?;
                
                match json_value.0 {
                    serde_json::Value::Null => Ok($name(None)),
                    other => {
                        match serde_json::from_value::<$generic>(other) {
                            Ok(parsed) => Ok($name(Some(parsed))),
                            Err(_) => Ok($name(None)),
                        }
                    }
                }
            }
        }

        impl<$generic> sqlx::Type<sqlx::Postgres> for $name<$generic> {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("jsonb")
            }
        }

    };
    // Handle non-generic types (original functionality)
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident($inner:ty)
    ) => {
        $(#[$attr])*
        $vis struct $name($inner);

        impl std::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }

        impl From<serde_json::Value> for $name {
            fn from(value: serde_json::Value) -> Self {
                Self(serde_json::from_value(value).unwrap_or_default())
            }
        }

        sqlx_transparent_json_decode::sqlx_json_decode!($name);
    };
}


/// Implement From<Option<JsonValue>> for JsonOption<T> for specific types
/// Usage: impl_json_option_from!(ModelParameters);
macro_rules! impl_json_option_from {
    ($concrete_type:ty) => {
        impl From<Option<serde_json::Value>> for crate::database::types::JsonOption<$concrete_type> {
            fn from(value: Option<serde_json::Value>) -> Self {
                crate::database::types::JsonOption::from_json_option(value)
            }
        }
    };
}


/// Implement From<String> for enums that have from_str() method
/// Usage: impl_string_to_enum!(EngineType);
/// This allows SQLx to automatically convert database strings to enum types
macro_rules! impl_string_to_enum {
    ($enum_type:ty) => {
        impl From<String> for $enum_type {
            fn from(s: String) -> Self {
                Self::from_str(&s).unwrap_or_else(|| {
                    panic!("Invalid enum value '{}' for type {}", s, std::any::type_name::<$enum_type>())
                })
            }
        }
        
        impl From<&str> for $enum_type {
            fn from(s: &str) -> Self {
                Self::from_str(s).unwrap_or_else(|| {
                    panic!("Invalid enum value '{}' for type {}", s, std::any::type_name::<$enum_type>())
                })
            }
        }
    };
}

pub(crate) use make_transparent;
pub(crate) use impl_json_option_from;
pub(crate) use impl_string_to_enum;