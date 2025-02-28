use std::marker::PhantomData;

use ark_ff::{BigInteger, PrimeField};
use sha3::{Digest, Keccak256};

#[derive(Clone, Debug, Copy)]
pub struct Transcript<K: HashTrait, F: PrimeField> {
    hash_func: K,
    _field: PhantomData<F>
}

impl <K: HashTrait, F: PrimeField> Transcript<K, F> {
    pub fn new(hash_func: K) -> Self {
        Self {
            hash_func,
            _field: PhantomData
        }
    }

    pub fn absorb(&mut self, data: &[u8]) {
        self.hash_func.append(data);
    }

    pub fn squeeze(&mut self) -> F {
        let squeezed_hash = self.hash_func.generate_hash();
        self.absorb(&squeezed_hash);
        F::from_be_bytes_mod_order(&squeezed_hash)
    }

    pub fn squeeze_n(&mut self, n: usize) -> Vec<F> {
        let mut squeezes = vec![];
        for _ in 0..n {
            let squeezed_hash = self.hash_func.generate_hash();
            self.absorb(&squeezed_hash);
            squeezes.push( F::from_be_bytes_mod_order(&squeezed_hash))
        }
        squeezes
    }
}


pub fn conv_poly_to_byte<F: PrimeField>(poly: &Vec<F>) -> Vec<u8> {
    poly.iter().map(|y| y.into_bigint().to_bytes_be()).collect::<Vec<_>>().concat()
}

pub trait HashTrait {
    fn append(&mut self, data: &[u8]);
    fn generate_hash(&mut self) -> Vec<u8>;
}

impl HashTrait for Keccak256 {
    fn append(&mut self, data: &[u8]) {
        self.update(data);
    }

    fn generate_hash(&mut self) -> Vec<u8> {
        let challenge = self.clone().finalize().to_vec();
        self.update(&challenge);
        challenge
    }
}
