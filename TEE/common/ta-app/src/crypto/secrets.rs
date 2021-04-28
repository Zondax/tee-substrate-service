pub const SR_SECRET: Option<&'static str> = option_env!("SR25519_SECRET");
pub const ED_SECRET: Option<&'static str> = option_env!("ED25519_SECRET");
pub const EC_SECRET: Option<&'static str> = option_env!("ECDSA_SECRET");

pub const SR_KEY_TYPES: Option<&'static str> = option_env!("SR25519_KEY_TYPES");
pub const ED_KEY_TYPES: Option<&'static str> = option_env!("ED25519_KEY_TYPES");
pub const EC_KEY_TYPES: Option<&'static str> = option_env!("ECDSA_KEY_TYPES");
