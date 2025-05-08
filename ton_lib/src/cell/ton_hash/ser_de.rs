#[cfg(feature = "serde_scylla")]
mod serde_scylla {
    use crate::cell::ton_hash::TonHash;
    use scylla::_macro_internal::{
        CellWriter, ColumnType, DeserializationError, DeserializeValue, FrameSlice, SerializationError, SerializeValue,
        TypeCheckError, WrittenCellProof,
    };
    use scylla::cluster::metadata::NativeType;
    use scylla::deserialize::value::{BuiltinTypeCheckError, BuiltinTypeCheckErrorKind};

    impl DeserializeValue<'_, '_> for TonHash {
        fn type_check(typ: &ColumnType) -> Result<(), TypeCheckError> {
            match typ {
                ColumnType::Native(NativeType::Blob) => Ok(()),
                _ => Err(TypeCheckError::new(BuiltinTypeCheckError {
                    rust_name: "TonHash",
                    cql_type: typ.clone().into_owned(),
                    kind: BuiltinTypeCheckErrorKind::MismatchedType {
                        expected: &[ColumnType::Native(NativeType::Text)],
                    },
                })),
            }
        }

        fn deserialize<'a>(typ: &'a ColumnType<'a>, v: Option<FrameSlice<'_>>) -> Result<Self, DeserializationError> {
            match TonHash::from_vec(DeserializeValue::deserialize(typ, v)?) {
                Ok(val) => Ok(val),
                Err(err) => Err(DeserializationError::new(err)),
            }
        }
    }

    impl SerializeValue for TonHash {
        fn serialize<'b>(
            &self,
            typ: &ColumnType,
            writer: CellWriter<'b>,
        ) -> Result<WrittenCellProof<'b>, SerializationError> {
            SerializeValue::serialize(&self.0.as_slice(), typ, writer)
        }
    }
}

pub mod serde_ton_hash_b64 {
    use crate::cell::ton_hash::TonHash;
    use serde::{de::Error, Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S: Serializer>(hash: &TonHash, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(hash.to_b64().as_str())
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<TonHash, D::Error> {
        TonHash::from_str(&String::deserialize(deserializer)?).map_err(Error::custom)
    }
}

pub mod serde_ton_hash_vec_b64 {
    use crate::cell::ton_hash::TonHash;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::str::FromStr;

    pub fn serialize<S: Serializer>(data: &[TonHash], serializer: S) -> Result<S::Ok, S::Error> {
        let b64_strings: Vec<String> = data.iter().map(|h| h.to_b64()).collect();
        b64_strings.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<TonHash>, D::Error> {
        let b64_vec: Vec<String> = Vec::deserialize(deserializer)?;
        b64_vec.into_iter().map(|s| TonHash::from_str(&s).map_err(serde::de::Error::custom)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::ton_hash::TonHash;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[test]
    fn test_ton_hash_serde() -> anyhow::Result<()> {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestStruct {
            #[serde(with = "serde_ton_hash_b64")]
            hash: TonHash,
            #[serde(with = "serde_ton_hash_vec_b64")]
            hash_vec: Vec<TonHash>,
        }

        let val = TestStruct {
            hash: TonHash::from_bytes(&[1u8; 32])?,
            hash_vec: vec![TonHash::from_bytes(&[2u8; 32])?, TonHash::from_bytes(&[3u8; 32])?],
        };
        let val_json = serde_json::to_string(&val)?;
        let expected = json!({
            "hash": "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=",
            "hash_vec": [
                "AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI=",
                "AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM="
            ]
        })
        .to_string();
        assert_eq!(val_json, expected);
        Ok(())
    }
}
