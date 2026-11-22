//! Record batch attributes with names: how a batch's timestamps were set, and how it
//! is compressed. Both also appear as topic configs (`message.timestamp.type`,
//! `compression.type`), by name.

use std::fmt;

use crate::LogSafe;

/// Who set a batch's timestamps.
///
/// Kafka's `NO_TIMESTAMP_TYPE` (id -1) belongs to message formats v0 and v1, which
/// Alluvium doesn't support, so it is not a variant: where it can appear, the field is
/// an `Option<TimestampType>` and it is `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimestampType {
    /// Id 0, `CreateTime`: the producer set them.
    CreateTime,
    /// Id 1, `LogAppendTime`: the broker set them when it appended the batch.
    LogAppendTime,
}

/// How a batch's records are compressed.
///
/// The topic config `compression.type` also accepts `producer` ("keep what the
/// producer used"). That is a config value, not a codec, so it belongs to the config
/// types, not here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressionType {
    /// Id 0, `none`.
    None,
    /// Id 1, `gzip`.
    Gzip,
    /// Id 2, `snappy`.
    Snappy,
    /// Id 3, `lz4`.
    Lz4,
    /// Id 4, `zstd`.
    Zstd,
}

/// An id or name that is not one of these enums' values.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AttributeError {
    /// A timestamp type id other than 0 or 1.
    #[error("unknown timestamp type id {id}")]
    UnknownTimestampTypeId {
        /// The rejected id.
        id: i32,
    },
    /// A timestamp type name other than `CreateTime` or `LogAppendTime`.
    #[error("unknown timestamp type \"{}\"", LogSafe(.name))]
    UnknownTimestampTypeName {
        /// The rejected name.
        name: String,
    },
    /// A compression type id outside 0 to 4.
    #[error("unknown compression type id {id}")]
    UnknownCompressionId {
        /// The rejected id.
        id: i8,
    },
    /// A compression type name other than `none`, `gzip`, `snappy`, `lz4` or `zstd`.
    #[error("unknown compression type \"{}\"", LogSafe(.name))]
    UnknownCompressionName {
        /// The rejected name.
        name: String,
    },
}

// Parity: clients/src/main/java/org/apache/kafka/common/record/TimestampType.java@4.4.0 TimestampType.id, TimestampType.forName
impl TimestampType {
    /// The type with Kafka's id `id`.
    ///
    /// # Errors
    ///
    /// [`AttributeError::UnknownTimestampTypeId`] for any id but 0 and 1, including
    /// `NO_TIMESTAMP_TYPE`'s -1.
    pub fn from_id(id: i32) -> Result<TimestampType, AttributeError> {
        match id {
            0 => Ok(TimestampType::CreateTime),
            1 => Ok(TimestampType::LogAppendTime),
            _ => Err(AttributeError::UnknownTimestampTypeId { id }),
        }
    }

    /// Kafka's id.
    pub fn id(self) -> i32 {
        match self {
            TimestampType::CreateTime => 0,
            TimestampType::LogAppendTime => 1,
        }
    }

    /// The type named `name`, matched exactly as Kafka's `forName` does.
    ///
    /// # Errors
    ///
    /// [`AttributeError::UnknownTimestampTypeName`] for any other name.
    pub fn from_name(name: &str) -> Result<TimestampType, AttributeError> {
        match name {
            "CreateTime" => Ok(TimestampType::CreateTime),
            "LogAppendTime" => Ok(TimestampType::LogAppendTime),
            _ => Err(AttributeError::UnknownTimestampTypeName {
                name: name.to_owned(),
            }),
        }
    }

    /// Kafka's name, `CreateTime` or `LogAppendTime`.
    pub fn name(self) -> &'static str {
        match self {
            TimestampType::CreateTime => "CreateTime",
            TimestampType::LogAppendTime => "LogAppendTime",
        }
    }
}

// Parity: clients/src/main/java/org/apache/kafka/common/record/internal/CompressionType.java@4.4.0 CompressionType.forId, CompressionType.forName
impl CompressionType {
    /// The codec with Kafka's id `id`, the low three bits of a batch's attributes.
    ///
    /// # Errors
    ///
    /// [`AttributeError::UnknownCompressionId`] for any id outside 0 to 4.
    pub fn from_id(id: i8) -> Result<CompressionType, AttributeError> {
        match id {
            0 => Ok(CompressionType::None),
            1 => Ok(CompressionType::Gzip),
            2 => Ok(CompressionType::Snappy),
            3 => Ok(CompressionType::Lz4),
            4 => Ok(CompressionType::Zstd),
            _ => Err(AttributeError::UnknownCompressionId { id }),
        }
    }

    /// Kafka's id.
    pub fn id(self) -> i8 {
        match self {
            CompressionType::None => 0,
            CompressionType::Gzip => 1,
            CompressionType::Snappy => 2,
            CompressionType::Lz4 => 3,
            CompressionType::Zstd => 4,
        }
    }

    /// The codec named `name`, matched exactly (so `GZIP` is unknown), as Kafka's
    /// `forName` does.
    ///
    /// # Errors
    ///
    /// [`AttributeError::UnknownCompressionName`] for any other name.
    pub fn from_name(name: &str) -> Result<CompressionType, AttributeError> {
        match name {
            "none" => Ok(CompressionType::None),
            "gzip" => Ok(CompressionType::Gzip),
            "snappy" => Ok(CompressionType::Snappy),
            "lz4" => Ok(CompressionType::Lz4),
            "zstd" => Ok(CompressionType::Zstd),
            _ => Err(AttributeError::UnknownCompressionName {
                name: name.to_owned(),
            }),
        }
    }

    /// Kafka's name: `none`, `gzip`, `snappy`, `lz4` or `zstd`.
    pub fn name(self) -> &'static str {
        match self {
            CompressionType::None => "none",
            CompressionType::Gzip => "gzip",
            CompressionType::Snappy => "snappy",
            CompressionType::Lz4 => "lz4",
            CompressionType::Zstd => "zstd",
        }
    }
}

impl fmt::Display for TimestampType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl fmt::Display for CompressionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::{AttributeError, CompressionType, TimestampType};

    const TIMESTAMP_TYPES: [TimestampType; 2] =
        [TimestampType::CreateTime, TimestampType::LogAppendTime];

    const COMPRESSION_TYPES: [CompressionType; 5] = [
        CompressionType::None,
        CompressionType::Gzip,
        CompressionType::Snappy,
        CompressionType::Lz4,
        CompressionType::Zstd,
    ];

    #[test]
    fn every_timestamp_type_round_trips_by_id_and_name() {
        for kind in TIMESTAMP_TYPES {
            assert_eq!(TimestampType::from_id(kind.id()), Ok(kind));
            assert_eq!(TimestampType::from_name(kind.name()), Ok(kind));
            assert_eq!(kind.to_string(), kind.name());
        }
        assert_eq!(
            TIMESTAMP_TYPES.map(TimestampType::name),
            ["CreateTime", "LogAppendTime"]
        );
        for id in [-1, 2, i32::MIN, i32::MAX] {
            assert_eq!(
                TimestampType::from_id(id),
                Err(AttributeError::UnknownTimestampTypeId { id })
            );
        }
        for name in ["NoTimestampType", "createtime", "", "CreateTime "] {
            assert!(TimestampType::from_name(name).is_err(), "{name:?}");
        }
    }

    #[test]
    fn every_compression_type_round_trips_by_id_and_name() {
        for codec in COMPRESSION_TYPES {
            assert_eq!(CompressionType::from_id(codec.id()), Ok(codec));
            assert_eq!(CompressionType::from_name(codec.name()), Ok(codec));
            assert_eq!(codec.to_string(), codec.name());
        }
        assert_eq!(COMPRESSION_TYPES.map(CompressionType::id), [0, 1, 2, 3, 4]);
        for id in i8::MIN..=i8::MAX {
            if !(0..=4).contains(&id) {
                assert_eq!(
                    CompressionType::from_id(id),
                    Err(AttributeError::UnknownCompressionId { id })
                );
            }
        }
        for name in ["GZIP", "producer", "lz5", "", "zstd\n"] {
            assert!(CompressionType::from_name(name).is_err(), "{name:?}");
        }
    }

    #[test]
    fn unknown_names_print_log_safe() {
        assert_eq!(
            CompressionType::from_name("lz4\nERROR forged")
                .unwrap_err()
                .to_string(),
            r#"unknown compression type "lz4\nERROR forged""#
        );
    }
}
