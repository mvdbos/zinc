use blake2_rfc::blake2b::Blake2b;
use blake2_rfc::blake2s::Blake2s;
use sha2::{Sha256, Digest};

use jubjub::{JubjubEngine, ToUniform};


pub fn hash_to_scalar<E: JubjubEngine>(persona: &[u8], a: &[u8], b: &[u8]) -> E::Fs {
    let mut hasher = Blake2b::with_params(64, &[], &[], persona);
    hasher.update(a);
    hasher.update(b);
    let ret = hasher.finalize();
    E::Fs::to_uniform(ret.as_ref())
}

pub fn hash_to_scalar_s<E: JubjubEngine>(persona: &[u8], a: &[u8], b: &[u8]) -> E::Fs {
    let mut hasher = Blake2s::with_params(32, &[], &[], persona);
    hasher.update(a);
    hasher.update(b);
    let ret = hasher.finalize();
    E::Fs::to_uniform_32(ret.as_ref())
}

pub fn sha256_hash_to_scalar<E: JubjubEngine>(persona: &[u8], a: &[u8], b: &[u8]) -> E::Fs {
    let mut hasher = Sha256::new();
    hasher.update(persona);
    hasher.update(a);
    hasher.update(b);
    let result = hasher.finalize();
    E::Fs::to_uniform_32(result.as_slice())
}