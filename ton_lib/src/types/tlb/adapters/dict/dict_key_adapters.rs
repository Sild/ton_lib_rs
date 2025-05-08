use crate::cell::ton_hash::TonHash;
use crate::errors::TonlibError;
use num_bigint::BigUint;

pub trait DictKeyAdapter<K> {
    fn make_key(src_key: &K) -> Result<BigUint, TonlibError>;
    fn extract_key(dict_key: &BigUint) -> Result<K, TonlibError>;
}

pub struct DictKeyAdapterTonHash; // properly tested in LibsDict
pub struct DictKeyAdapterInto;
pub struct DictKeyAdapterString; // TODO is not covered by tests

impl DictKeyAdapter<TonHash> for DictKeyAdapterTonHash {
    fn make_key(src_key: &TonHash) -> Result<BigUint, TonlibError> { Ok(BigUint::from_bytes_be(src_key.as_slice())) }

    fn extract_key(dict_key: &BigUint) -> Result<TonHash, TonlibError> {
        let mut hash_bytes = vec![0; TonHash::BYTES_LEN];
        let key_bytes = dict_key.to_bytes_le();
        if key_bytes.len() > TonHash::BYTES_LEN {
            return Err(TonlibError::TLBDictWrongKeyLen {
                exp: TonHash::BYTES_LEN,
                got: key_bytes.len(),
                key: dict_key.clone(),
            });
        }
        let offset = TonHash::BYTES_LEN - key_bytes.len();
        hash_bytes.as_mut_slice()[offset..].copy_from_slice(key_bytes.as_slice());
        TonHash::from_bytes(&hash_bytes)
    }
}

impl<T: Clone + Into<BigUint> + TryFrom<BigUint>> DictKeyAdapter<T> for DictKeyAdapterInto {
    fn make_key(src_key: &T) -> Result<BigUint, TonlibError> { Ok(src_key.clone().into()) }

    fn extract_key(dict_key: &BigUint) -> Result<T, TonlibError> {
        match T::try_from(dict_key.clone()) {
            Ok(key) => Ok(key),
            Err(_) => {
                // TODO cleanup error handling
                Err(TonlibError::CustomError("fail to extract dict key".to_string()))
            }
        }
    }
}

impl DictKeyAdapter<String> for DictKeyAdapterString {
    fn make_key(src_key: &String) -> Result<BigUint, TonlibError> {
        let bytes = src_key.as_bytes();
        Ok(BigUint::from_bytes_le(bytes))
    }

    fn extract_key(dict_key: &BigUint) -> Result<String, TonlibError> {
        let bytes = dict_key.to_bytes_le();
        Ok(String::from_utf8(bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_dict_key_adapter_ton_hash() -> anyhow::Result<()> {
        let dict_key = DictKeyAdapterTonHash::make_key(&TonHash::ZERO)?;
        assert_eq!(dict_key, 0u32.into());
        assert_eq!(DictKeyAdapterTonHash::extract_key(&dict_key)?, TonHash::ZERO);

        let dict_key = DictKeyAdapterTonHash::make_key(&TonHash::from([0b1010_1010; 32]))?;
        assert_eq!(
            dict_key,
            BigUint::from_str("77194726158210796949047323339125271902179989777093709359638389338608753093290")?
        );
        assert_eq!(DictKeyAdapterTonHash::extract_key(&dict_key)?, TonHash::from([0b1010_1010; 32]));
        Ok(())
    }
}
