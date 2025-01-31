use ark_ff::{BigInteger, PrimeField};
use std::marker::PhantomData;
use sha3::{Digest, Keccak256};
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
// 2. uses its own transcript to generate random challenge and evalauates the univariate polynomial at that random challenge. This gives the claimed sum of the next polynomial.
// 3. compares these two sums then evaluates the initial polynomial at the random challenge for use in the next round. The cycle repeats from step 2

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
        self.absorb(&squeezed_hash);
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

    fn generate_proof(&mut self, evals: Vec<F>) -> Proof<F> {
        let mut proof_array = vec![];
        let mut current_poly = evals.clone();
        let mut rand_chal;
        let no_of_vars = (evals.len() as f64).log2();
        for _ in 0..no_of_vars as usize {
            let (sum, univar_poly) = self.generate_sum_and_univariate_poly(&current_poly);
            proof_array.push((sum, univar_poly.clone())); // for verifier to be able to access it later
            self.transcript.absorb(sum.into_bigint().to_bytes_be().as_slice());
            self.transcript.absorb(&univar_poly.iter().map(|y| y.into_bigint().to_bytes_be()).collect::<Vec<_>>().concat());
            rand_chal = self.transcript.squeeze();
            current_poly = eval_interpol::evaluate_interpolate(current_poly.clone(), 0, rand_chal);

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
    fn init(polynomial: Vec<F>, transcript: Transcript<Keccak256, F>, proof: Proof<F>) -> Self {
        Self { 
            polynomial,
            transcript,
            proof
        }
    }

    fn verify(&mut self) -> bool {
        let proof = &self.proof.univars_and_sums;
        println!("proof length {:?}", proof.len());
        println!("proof {:?}", proof);
        let mut rand_chal_array: Vec<F> = vec![];

        for i in 0..proof.len() {
            let (sum, univar_poly) = &proof[i];
            let mut claimed_sum: F = *sum;
            println!("univar poly {:?}", univar_poly);
            println!("sum {:?}", sum);
            let claim: F = univar_poly.iter().sum();
            assert_eq!(claimed_sum, claim);
            self.transcript.absorb(sum.into_bigint().to_bytes_be().as_slice());
            self.transcript.absorb(&univar_poly.iter().map(|y| y.into_bigint().to_bytes_be()).collect::<Vec<_>>().concat());
            let rand_chal = self.transcript.squeeze();
            rand_chal_array.push(rand_chal);
            let current_poly = eval_interpol::evaluate_interpolate(univar_poly.clone(), 0, rand_chal);
            claimed_sum = current_poly[0];

            if i + 1 < proof.len() {
                assert_eq!(claimed_sum, proof[i + 1].0)
            }

            if i == proof.len() - 1 {
                // do oracle check
                // let mut final_check_sum: Vec<F> = vec![];
                let mut poly: Vec<F> = self.polynomial.clone();

                println!("rand chal array {:?}", rand_chal_array);
                for i in 0..rand_chal_array.len() {
                    poly = eval_interpol::evaluate_interpolate(poly.clone(), 0, rand_chal_array[i]);
                }
                println!("final sum {:?}", poly[0]);
                println!("final pol {:?}", current_poly[0]);
                assert_eq!(poly[0], current_poly[0])
            }
        }
        true
    }
}

// #[cfg(test)]
// mod test {
//     use super::{Prover, Verifier, Transcript, Keccak256};
//     use sha3::Digest;
//     use ark_bn254::Fq;

//     // #[test]
//     // fn test_transcript() {
//     //     let mut transcript = Transcript::<Keccak256, Fq>::new(Keccak256::new());

//     //     transcript.absorb(b"[0,1]");
//     //     transcript.absorb(b"50");

//     //     let random_chal = transcript.squeeze();
//     //     println!("{}", random_chal);
//     // }

//     #[test]
//     fn test_generate_proof() {
//         let transcript = Transcript::<Keccak256, Fq>::new(Keccak256::new());
//         let evals = vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(2), Fq::from(4)];
//         let mut prover = Prover::init(evals.clone(), transcript);
//         let proof = prover.generate_proof(evals);

//         println!("{:?}", proof);
//     }

//     #[test]
//     fn test_verifier() {
//         let evals = vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(2), Fq::from(4)];
//         let transcript = Transcript::<Keccak256, Fq>::new(Keccak256::new());
//         let mut prover = Prover::init(evals.clone(), transcript);
//         let proof = prover.generate_proof(evals.clone());

//         let mut verifier = Verifier::init(evals.clone(), Transcript::<Keccak256, Fq>::new(Keccak256::new()), proof);
//         let result = verifier.verify();
//         assert!(result, "{}", true);
//     }
// }