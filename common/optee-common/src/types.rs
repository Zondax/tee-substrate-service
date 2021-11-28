use super::*;

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[repr(u32)]
///Represents command to send to the tee
pub enum CommandId {
    GenerateNew,
    GetKeys,
    SignMessage,
    HasKeys,
    VrfSign,
}

impl std::convert::TryFrom<u32> for CommandId {
    type Error = ();

    fn try_from(cmd: u32) -> Result<Self, ()> {
        match cmd {
            0 => Ok(CommandId::GenerateNew),
            1 => Ok(CommandId::GetKeys),
            2 => Ok(CommandId::SignMessage),
            3 => Ok(CommandId::HasKeys),
            4 => Ok(CommandId::VrfSign),
            _ => Err(()),
        }
    }
}

impl Into<u32> for CommandId {
    fn into(self) -> u32 {
        self as _
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
///Represents the type of algorithm to use for the key
pub enum CryptoAlgo {
    Sr25519,
    Ed25519,
    Ecdsa
}

impl Into<u8> for CryptoAlgo {
    fn into(self) -> u8 {
        self as _
    }
}

impl CryptoAlgo {
    pub const fn pubkey_len(&self) -> usize {
        match self {
            CryptoAlgo::Sr25519 => 32, //sp_core::sr25519::Public
            CryptoAlgo::Ed25519 => 32, //sp_core::ed25519::Public
            CryptoAlgo::Ecdsa => 33, //sp_core::ecdsa::Public
        }
    }

    pub const fn signature_len(&self) -> usize {
        match self {
            CryptoAlgo::Sr25519 => 64,
            CryptoAlgo::Ed25519 => 64,
            CryptoAlgo::Ecdsa => 65,
        }
    }
}

#[cfg(feature = "alloc")]
mod haskeyspair {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct HasKeysPair {
        pub key_type: [u8; 4],
        pub public_key: Vec<u8>,
    }
}

#[cfg(feature = "alloc")]
pub use haskeyspair::HasKeysPair;
