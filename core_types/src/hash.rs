use multihash::{encode, Hash};
use rust_base58::ToBase58;
use serde::Serialize;
use serde_json;
use std::fmt;

// HashString newtype for String
#[derive(PartialOrd, PartialEq, Eq, Ord, Clone, Debug, Serialize, Deserialize, Default, Hash)]
pub struct HashString(String);

impl fmt::Display for HashString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for HashString {
    fn from(s: String) -> HashString {
        HashString(s)
    }
}

impl From<HashString> for String {
    fn from(h: HashString) -> String {
        h.0
    }
}

impl<'a> From<&'a str> for HashString {
    fn from(s: &str) -> HashString {
        HashString::from(s.to_string())
    }
}

impl HashString {
    pub fn new() -> HashString {
        HashString("".to_string())
    }

    /// convert bytes to a b58 hashed string
    pub fn encode_from_bytes(bytes: &[u8], hash_type: Hash) -> HashString {
        HashString::from(encode(hash_type, bytes).unwrap().to_base58())
    }

    /// convert a string as bytes to a b58 hashed string
    pub fn encode_from_str(s: &str, hash_type: Hash) -> HashString {
        HashString::encode_from_bytes(s.as_bytes(), hash_type)
    }

    /// magic all in one fn, take a serializable something + hash type and get a hashed b58 string back
    pub fn encode_from_serializable<S: Serialize>(s: S, hash_type: Hash) -> HashString {
        HashString::encode_from_str(&serde_json::to_string(&s).unwrap(), hash_type)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use cas::content::AddressableContent;
    use entry::{test_app_entry};
    use multihash::Hash;
    use entry::expected_app_entry_content;

    /// dummy hash based on the key of test_app_entry()
    pub fn test_hash() -> HashString {
        test_app_entry().address()
    }

    #[test]
    /// show ToString implementation
    /// automatically derived by Rust because fmt::Display is implemented
    fn to_string_test() {
        assert_eq!(test_hash().to_string(), expected_app_entry_content().to_string(),)
    }

    #[test]
    /// show From<String> implementation
    fn from_string_test() {
        assert_eq!(HashString::new(), HashString::from("".to_string()),);

        assert_eq!(
            test_hash(),
            HashString::from(test_app_entry().address().to_string()),
        );
    }

    #[test]
    /// show From<&str> implementation
    fn from_str_test() {
        assert_eq!(HashString::new(), HashString::from(""));

        assert_eq!(test_hash(), HashString::from(test_app_entry().address()),);
    }

    #[test]
    /// mimics tests from legacy golang holochain core hashing bytes
    fn bytes_to_b58_known_golang() {
        assert_eq!(
            HashString::encode_from_bytes(b"test data", Hash::SHA2256).to_string(),
            "QmY8Mzg9F69e5P9AoQPYat655HEhc1TVGs11tmfNSzkqh2"
        )
    }

    #[test]
    /// mimics tests from legacy golang holochain core hashing strings
    fn str_to_b58_hash_known_golang() {
        assert_eq!(
            HashString::encode_from_str("test data", Hash::SHA2256).to_string(),
            "QmY8Mzg9F69e5P9AoQPYat655HEhc1TVGs11tmfNSzkqh2"
        );
    }

    #[test]
    /// known hash for a serializable something
    fn can_serialize_to_b58_hash() {
        #[derive(Serialize)]
        struct Foo {
            foo: u8,
        };

        assert_eq!(
            "Qme7Bu4NVYMtpsRtb7e4yyhcbE1zdB9PsrKTdosaqF3Bu3",
            HashString::encode_from_serializable(Foo { foo: 5 }, Hash::SHA2256).to_string(),
        );
    }
}
