use anyhow::{Context, Result, anyhow, bail};
use p256::{SecretKey, pkcs8::EncodePrivateKey};
use ring::{
    hkdf::{HKDF_SHA256, KeyType, Salt},
    rand::SystemRandom,
    signature::{self, EcdsaKeyPair},
};

const DOMAIN: &[u8] = b"sebastianvogelsang.com-mpds-demo-v1";
const P256_KEYLEN: usize = 32;

pub struct Keygen {
    salt: Salt,
}

struct P256;

impl KeyType for P256 {
    fn len(&self) -> usize {
        P256_KEYLEN
    }
}

impl Keygen {
    pub fn new() -> Self {
        let salt = Salt::new(HKDF_SHA256, DOMAIN);
        Self { salt }
    }

    pub fn generate(&self, prf: &[u8], did: &[u8]) -> Result<EcdsaKeyPair> {
        let mut key = [0u8; P256_KEYLEN];
        let prk = self.salt.extract(prf);
        let rng = SystemRandom::new();

        for n in 0..8 {
            let info = [b"round".as_ref(), &[n], b"did".as_ref(), &did];

            let okm = prk
                .expand(&info, P256)
                .map_err(|e| anyhow!("failed to expand key: {}", e))?;

            okm.fill(&mut key)
                .map_err(|e| anyhow!("failed to fill key buffer: {}", e))?;

            if key == [0u8; P256_KEYLEN] {
                continue;
            }

            let key = SecretKey::from_bytes(&key.into()).context("malformed key")?;
            let key = key
                .to_pkcs8_der()
                .context("failed to convert key to PKCS8 DER")?;

            if let Ok(keypair) = EcdsaKeyPair::from_pkcs8(
                &signature::ECDSA_P256_SHA256_FIXED_SIGNING,
                key.as_bytes(),
                &rng,
            ) {
                return Ok(keypair);
            } else {
                continue;
            }
        }

        bail!("cannot generate valid key, giving up");
    }
}
