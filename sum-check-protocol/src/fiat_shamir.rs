use ark_ff::PrimeField;
use std::marker::PhantomData;
use sha3::{Keccak256, Digest};
pub mod eval_interpol;
// I need a prover, the fiat shamir transcript and the verifier
// prover needs to:
// 1. Evaluate a polynomial over the boolean hypercube and sum the result
// 2. Partially evaluate a polynomial at a particular var over the hypercube
// 3. Partially evaluate at one var only
// Prover sends proof containing the claimed sum, all polynomials evaluated in each step and their
// associated claimed sum

// Transcript:
// 1. gets polynomial and claimed sum from prover
// 2. hashes both by first absorbing each and then squeezing
// 3. sends new field element as challenge to prover

// Verifier:
// 1. gets proof from prover
// 2. gets the hash of the initial polynomial with the initial claimed sum from the transcript. With which
// it can get the random challenge after getting the random hash what happens?

struct Transcript<K: HashTrait, F: PrimeField> {
    hash_func: K,
    _field: PhantomData<F>
}

impl <K: HashTrait, F: PrimeField> Transcript<K, F> {
    fn new(hash_func: K) -> Self {
        Self {
            hash_func,
            _field: PhantomData
        }
    }

    fn absorb(&mut self, data: &[u8]) {
        self.hash_func.append(data);
    }

    fn squeeze(&self) -> F {
        let squeezed_hash = self.hash_func.generate_hash();
        F::from_be_bytes_mod_order(&squeezed_hash)
    }
}

trait HashTrait {
    fn append(&mut self, data: &[u8]);
    fn generate_hash(&self) -> Vec<u8>;
}

impl HashTrait for Keccak256 {
    fn append(&mut self, data: &[u8]) {
        self.update(data);
    }

    fn generate_hash(&self) -> Vec<u8> {
        self.clone().finalize().to_vec()
    }
}

struct Prover<F: PrimeField> {
    polynomial: Vec<F>,
    transcript: Transcript<Keccak256, F>
}

impl <F: PrimeField> Prover<F> {
    fn init(polynomial: Vec<F>) -> Self {
        Self { polynomial }
    }

    fn generate_sum_and_univariate_poly(no_of_vars: u32, evals: Vec<F>, var_index: usize, var_eval_at: F) -> (F, Vec<F>) {
        let claimed_sum = evals.iter().sum();
        let univar_poly = eval_interpol::evaluate_interpolate(no_of_vars, evals, var_index, var_eval_at);
        (claimed_sum, univar_poly)
    }

    fn init_transcript() -> Transcript<Keccak256, F> {
        Transcript::new(Keccak256::new())
    }
}

struct Proof<F: PrimeField> {
    univariates: Vec<F>
}

struct Verifier<F: PrimeField> {
    polynomial: Vec<F>,
    transcript: Transcript<Keccak256, F>,
    proof: Proof<F>
}

impl<F: PrimeField> Verifier<F> {
    fn init_transcript() -> Transcript<Keccak256, F> {
        Transcript::new(Keccak256::new())
    }
}

#[cfg(test)]
mod test {
    use super::Transcript;
    use super::Keccak256;
    use sha3::Digest;
    use ark_bn254::Fq;

    #[test]
    fn test_transcript() {
        let mut transcript = Transcript::<Keccak256, Fq>::new(Keccak256::new());

        transcript.absorb(b"[0,1]");
        transcript.absorb(b"50");

        let random_chal = transcript.squeeze();
        println!("{}", random_chal);
    }
}