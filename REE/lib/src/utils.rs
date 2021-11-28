use optee_common::{CryptoAlgo as CryptoAlgo2, HasKeysPair as HasKeysPair2};
use zkms_common::{CryptoAlgo, HasKeysPair};

pub(crate) fn convert_crypto_algo_to_optee(from: CryptoAlgo) -> CryptoAlgo2 {
    match from {
        CryptoAlgo::Sr25519 => CryptoAlgo2::Sr25519,
        CryptoAlgo::Ed25519 => CryptoAlgo2::Ed25519,
        CryptoAlgo::Ecdsa => CryptoAlgo2::Ecdsa,
    }
}

pub(crate) fn convert_crypto_algo_to_zkms(from: CryptoAlgo2) -> CryptoAlgo {
    match from {
        CryptoAlgo2::Sr25519 => CryptoAlgo::Sr25519,
        CryptoAlgo2::Ed25519 => CryptoAlgo::Ed25519,
        CryptoAlgo2::Ecdsa => CryptoAlgo::Ecdsa,
    }
}

pub(crate) fn convert_haskeys_to_optee(
    HasKeysPair {
        key_type,
        public_key,
    }: HasKeysPair,
) -> HasKeysPair2 {
    HasKeysPair2 {
        key_type,
        public_key,
    }
}

pub(crate) fn convert_haskeys_to_zkms(
    HasKeysPair2 {
        key_type,
        public_key,
    }: HasKeysPair2,
) -> HasKeysPair {
    HasKeysPair {
        key_type,
        public_key,
    }
}
