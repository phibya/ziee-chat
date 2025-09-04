use crate::database::macros::make_transparent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Custom wrapper for optional JSON fields that handles null values properly
make_transparent!(
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct JsonOption<T>(Option<T>)
);

impl<T> JsonOption<T> {
    /// Convert JsonOption<T> to Option<T>
    pub fn into_option(self) -> Option<T> {
        self.0
    }

    /// Convert &JsonOption<T> to Option<&T>
    pub fn as_option(&self) -> Option<&T> {
        self.0.as_ref()
    }

    /// Convert &JsonOption<T> to Option<T> by cloning
    pub fn to_option(&self) -> Option<T>
    where
        T: Clone,
    {
        self.0.clone()
    }
}

// Custom wrapper for optional enum fields that handles database string conversion
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnumOption<T>(Option<T>);

impl<T> EnumOption<T> {
    /// Create a new EnumOption
    pub fn new(value: Option<T>) -> Self {
        Self(value)
    }

    /// Convert EnumOption<T> to Option<T>
    pub fn into_option(self) -> Option<T> {
        self.0
    }

    /// Convert &EnumOption<T> to Option<&T>
    pub fn as_option(&self) -> Option<&T> {
        self.0.as_ref()
    }

    /// Convert &EnumOption<T> to Option<T> by cloning
    pub fn to_option(&self) -> Option<T>
    where
        T: Clone,
    {
        self.0.clone()
    }
}

impl<T> From<Option<T>> for EnumOption<T> {
    fn from(value: Option<T>) -> Self {
        Self(value)
    }
}

impl<T> From<EnumOption<T>> for Option<T> {
    fn from(value: EnumOption<T>) -> Self {
        value.0
    }
}

impl<T> std::ops::Deref for EnumOption<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for EnumOption<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Default implementation
impl<T> Default for EnumOption<T> {
    fn default() -> Self {
        Self(None)
    }
}
