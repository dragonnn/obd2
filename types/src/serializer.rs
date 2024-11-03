use alloc::{format, vec::Vec};
use core::marker::PhantomData;
use serde::{Deserialize, Serialize};
use serde_encrypt::{serialize::TypedSerialized, Error};

/// [postcard](https://docs.rs/postcard) serializer
#[derive(Debug)]
pub struct PostcardSerializer<T> {
    serialized: Vec<u8>,
    _type: PhantomData<T>,
}

impl<T> TypedSerialized for PostcardSerializer<T> {
    type T = T;

    fn new(serialized: Vec<u8>) -> Self
    where
        Self: Sized,
    {
        Self {
            serialized,
            _type: PhantomData::default(),
        }
    }

    fn as_slice(&self) -> &[u8] {
        &self.serialized
    }

    fn into_vec(self) -> Vec<u8> {
        self.serialized
    }

    /// # Failures
    ///
    /// - [SerializationError](serde_encrypt_core::error::ErrorKind::SerializationError) when failed to serialize message.
    fn serialize(v: &Self::T) -> Result<Self, Error>
    where
        Self: Sized,
        Self::T: Serialize,
    {
        let serialized = postcard::to_allocvec_crc32(v, crate::CRC.digest()).map_err(|e| {
            Error::serialization_error(&format!("failed to serialize data by postcard: {:?}", e))
        })?;
        Ok(Self::new(serialized))
    }

    /// # Failures
    ///
    /// - [DeserializationError](serde_encrypt_core::error::ErrorKind::DeserializationError) when failed to deserialize decrypted message.
    fn deserialize<'de>(&'de self) -> Result<Self::T, Error>
    where
        Self::T: Deserialize<'de>,
    {
        postcard::from_bytes_crc32(self.as_slice(), crate::CRC.digest()).map_err(|e| {
            Error::deserialization_error(&format!(
                "error on postcard deserialization after decryption: {:?}",
                e
            ))
        })
    }
}
