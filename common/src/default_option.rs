use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};

use std::ops::Deref;

#[derive(Debug, Clone)]
enum DefaultOptionValue<T> {
    Value(T),
    Default(T),
}

#[derive(Debug, Clone)]
pub struct DefaultOption<T>
where
    T: Serialize + DeserializeOwned,
{
    value: DefaultOptionValue<T>,
}

impl<T> DefaultOption<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub fn with_value(value: T) -> Self {
        DefaultOption { value: DefaultOptionValue::Value(value) }
    }

    pub fn with_default(default: T) -> Self {
        DefaultOption { value: DefaultOptionValue::Default(default) }
    }

    pub fn is_default(&self) -> bool {
        match self.value {
            DefaultOptionValue::Default(_) => true,
            _ => false,
        }
    }

    pub fn is_value(&self) -> bool {
        !self.is_default()
    }

    pub fn get(&self) -> &T {
        match &self.value {
            DefaultOptionValue::Value(value) => &value,
            DefaultOptionValue::Default(default) => &default,
        }
    }

    pub fn set(&mut self, value: T) {
        self.value = DefaultOptionValue::Value(value);
    }
}

impl<T> Serialize for DefaultOption<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.value {
            DefaultOptionValue::Value(value) => value,
            DefaultOptionValue::Default(default) => default,
        }
        .serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for DefaultOption<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer).map(|x| DefaultOption { value: DefaultOptionValue::Value(x) })
    }
}

impl<T> Deref for DefaultOption<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    type Target = T;

    fn deref(&self) -> &T {
        self.get()
    }
}
