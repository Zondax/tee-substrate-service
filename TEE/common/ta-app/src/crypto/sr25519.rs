use schnorrkel::keys;

use crate::util::CSPRNG;

#[derive(Clone)]
pub struct Keypair(keys::Keypair);

impl Keypair {
    pub fn generate<C: CSPRNG>(rng: &mut C) -> Self {
        Self(keys::Keypair::generate_with(rng))
    }

    pub fn public(&self) -> &[u8] {
        self.0.public.as_ref()
    }

    pub fn sign<C: CSPRNG>(&self, rng: &mut C, msg: &[u8]) -> [u8; 64] {
        let mut t = merlin::Transcript::new(b"SigningContext");
        t.append_message(b"", b"substrate"); //ctx
        t.append_message(b"sign-bytes", msg);

        let t = schnorrkel::context::attach_rng(t, rng);

        self.0.sign(t).to_bytes()
    }
}
