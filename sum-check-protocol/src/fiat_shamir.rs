use ark_ff::{BigInteger, PrimeField};
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

    fn squeeze(&mut self) -> F {
        let squeezed_hash = self.hash_func.generate_hash();
        F::from_be_bytes_mod_order(&squeezed_hash)
    }
}

trait HashTrait {
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

struct Prover<F: PrimeField> {
    polynomial: Vec<F>,
    transcript: Transcript<Keccak256, F>
}

impl <F: PrimeField> Prover<F> {
    fn init(polynomial: Vec<F>, transcript: Transcript<Keccak256, F>) -> Self {
        Self { 
            polynomial,
            transcript
         }
    }

    fn generate_sum_and_univariate_poly(&mut self, evals: &[F]) -> (F, Vec<F>) {
        let claimed_sum = evals.iter().sum();
        let evals_len = evals.len() / 2;
        let mut univar_poly = vec![];
        univar_poly.push(evals.iter().take(evals_len).sum());
        univar_poly.push(evals.iter().skip(evals_len).sum());
        (claimed_sum, univar_poly)
    }

    fn generate_proof(&mut self, no_of_vars: u32, evals: Vec<F>) -> Proof<F> {
        let mut proof_array = vec![];
        let mut current_poly = evals;
        let mut rand_chal;
        for i in 0..no_of_vars {
            let (sum, univar_poly) = self.generate_sum_and_univariate_poly(&current_poly);
            proof_array.push((sum, univar_poly.clone())); // for verifier to be able to access it later
            self.transcript.absorb(sum.into_bigint().to_bytes_be().as_slice());
            self.transcript.absorb(&univar_poly.iter().map(|y| y.into_bigint().to_bytes_be()).collect::<Vec<_>>().concat());
            rand_chal = self.transcript.squeeze();
            current_poly = eval_interpol::evaluate_interpolate(no_of_vars, current_poly.clone(), i.try_into().unwrap(), rand_chal);
        }

        Proof {
            univars_and_sums: proof_array
        }
    }
}

#[derive(Debug)]
struct Proof<F: PrimeField> {
    univars_and_sums: Vec<(F, Vec<F>)>
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
    use super::Prover;
    use super::Transcript;
    use super::Keccak256;
    use sha3::Digest;
    use ark_bn254::Fq;

    // #[test]
    // fn test_transcript() {
    //     let mut transcript = Transcript::<Keccak256, Fq>::new(Keccak256::new());

    //     transcript.absorb(b"[0,1]");
    //     transcript.absorb(b"50");

    //     let random_chal = transcript.squeeze();
    //     println!("{}", random_chal);
    // }

    #[test]
    fn test_generate_proof() {
        let transcript = Transcript::<Keccak256, Fq>::new(Keccak256::new());
        let evals = vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(2), Fq::from(4)];
        let mut prover = Prover::init(evals.clone(), transcript);
        let proof = prover.generate_proof(3, evals);

        println!("{:?}", proof);
    }
}