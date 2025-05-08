use crate::types::ton_address::TonAddress;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

impl Serialize for TonAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for TonAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        TonAddress::from_str(&str).map_err(serde::de::Error::custom)
    }
}

pub mod serde_ton_address_hex {
    use crate::types::ton_address::TonAddress;
    use serde::{de::Error, Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S: Serializer>(hash: &TonAddress, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(hash.to_hex().as_str())
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<TonAddress, D::Error> {
        TonAddress::from_str(&String::deserialize(deserializer)?).map_err(Error::custom)
    }
}

#[cfg(feature = "serde_scylla")]
mod serde_scylla {
    use crate::types::ton_address::TonAddress;
    use scylla::_macro_internal::{
        CellWriter, ColumnType, DeserializationError, DeserializeValue, FrameSlice, SerializationError, SerializeValue,
        TypeCheckError, WrittenCellProof,
    };
    use scylla::cluster::metadata::NativeType;
    use scylla::deserialize::value::{BuiltinTypeCheckError, BuiltinTypeCheckErrorKind};
    use std::str::FromStr;

    impl DeserializeValue<'_, '_> for TonAddress {
        fn type_check(typ: &ColumnType) -> Result<(), TypeCheckError> {
            match typ {
                ColumnType::Native(NativeType::Text) => Ok(()),
                _ => Err(TypeCheckError::new(BuiltinTypeCheckError {
                    rust_name: "TonAddress",
                    cql_type: typ.clone().into_owned(),
                    kind: BuiltinTypeCheckErrorKind::MismatchedType {
                        expected: &[ColumnType::Native(NativeType::Text)],
                    },
                })),
            }
        }

        fn deserialize<'a>(typ: &'a ColumnType<'a>, v: Option<FrameSlice<'_>>) -> Result<Self, DeserializationError> {
            let address_str: String = DeserializeValue::deserialize(typ, v)?;
            match TonAddress::from_str(&address_str) {
                Ok(val) => Ok(val),
                Err(err) => Err(DeserializationError::new(err)),
            }
        }
    }

    impl SerializeValue for TonAddress {
        fn serialize<'b>(
            &self,
            typ: &ColumnType,
            writer: CellWriter<'b>,
        ) -> Result<WrittenCellProof<'b>, SerializationError> {
            SerializeValue::serialize(&self.to_string(), typ, writer)
        }
    }
}
