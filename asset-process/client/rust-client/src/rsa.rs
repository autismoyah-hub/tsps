//! The client uses raw RSA, with Java BigInteger's signed byte representation.
//! This type accepts public keys only and never handles a private exponent.
use crate::{writer::Writer, Error, Result};
use num_bigint::{BigInt, Sign};

pub struct RsaPublicKey {
    modulus: BigInt,
    exponent: BigInt,
}
impl RsaPublicKey {
    pub fn from_hex(modulus: &str, exponent: u32) -> Result<Self> {
        if modulus.is_empty()
            || modulus.len() > 1024
            || !modulus.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(Error::Invalid("RSA modulus hex"));
        }
        let modulus =
            BigInt::parse_bytes(modulus.as_bytes(), 16).ok_or(Error::Invalid("RSA modulus"))?;
        if modulus <= BigInt::from(1)
            || (&modulus & BigInt::from(1)) != BigInt::from(1)
            || exponent < 3
            || exponent & 1 == 0
        {
            return Err(Error::Invalid("RSA public key"));
        }
        Ok(Self {
            modulus,
            exponent: BigInt::from(exponent),
        })
    }
    pub fn encrypt_java_signed(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        if plaintext.is_empty() || plaintext.len() > 512 {
            return Err(Error::Invalid("RSA plaintext size"));
        }
        let input = BigInt::from_signed_bytes_be(plaintext);
        if input.sign() != Sign::Minus && input >= self.modulus {
            return Err(Error::Invalid("RSA plaintext exceeds modulus"));
        }
        Ok(input
            .modpow(&self.exponent, &self.modulus)
            .to_signed_bytes_be())
    }
    /// Matches xm.dd: unsigned-short ciphertext length followed by Java bytes.
    pub fn encode_block(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let encrypted = self.encrypt_java_signed(plaintext)?;
        let mut w = Writer::new();
        w.u16(encrypted.len() as u16);
        w.bytes(&encrypted);
        Ok(w.into_vec())
    }
}
