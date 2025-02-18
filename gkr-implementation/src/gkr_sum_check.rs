// I need to import the transcript from the fiat shamir implementation
// I need to modify the prover and verify. Instead if them working with a single polynomial, they would now work with an array of polynomials
// Prover:
// It now takes in a vector that takes in two productPoly structs 
// and each productpoly contains two array of evaluations. We need to be able to do sum check on each 
// vec in the array of evals. You know the prev sum check protocol only took in one array of evals
fn main() {
    // println!("Hello, world!");
}
// Verifier:
use ark_ff::{BigInteger, PrimeField};
use sha3::{Keccak256};
use crate::bit_format;
use crate::univar_poly;
use num_bigint::ToBigInt;

// use zk_polynomials::UnivariatePoly;
// use zk_polynomials::UnivariatePoly;

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

pub struct Prover<F: PrimeField> {
    poly_vec: Vec<bit_format::ProductPoly<F>>,
    transcript: bit_format::Transcript<Keccak256, F>
}

impl <F: PrimeField> Prover<F> {
    pub fn init(poly_vec: Vec<bit_format::ProductPoly<F>>, transcript: bit_format::Transcript<Keccak256, F>) -> Self {
        Self { 
            poly_vec,
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

    // pub fn generate_proof(&mut self, evals: Vec<F>) -> Proof<F> {
    //     let mut proof_array = vec![];
    //     let mut current_poly = evals.clone();
    //     let mut rand_chal;
    //     self.transcript.absorb(&evals.iter().map(|y| y.into_bigint().to_bytes_be()).collect::<Vec<_>>().concat());
    //     let no_of_vars = (evals.len() as f64).log2();
    //     for _ in 0..no_of_vars as usize {
    //         let (sum, univar_poly) = self.generate_sum_and_univariate_poly(&current_poly);
    //         proof_array.push((sum, univar_poly.clone())); // for verifier to be able to access it later
    //         self.transcript.absorb(sum.into_bigint().to_bytes_be().as_slice());
    //         self.transcript.absorb(&univar_poly.iter().map(|y| y.into_bigint().to_bytes_be()).collect::<Vec<_>>().concat());
    //         rand_chal = self.transcript.squeeze();
    //         current_poly = bit_format::evaluate_interpolate(current_poly.clone(), 0, rand_chal);

    //     }

    //     Proof {
    //         initial_claimed_sum,
    //         univars_and_sums: proof_array
    //     }
    // }

    // [ProductPoly { polys: [[0, 1, 0, 0], [30, 1695, 1695, 3360]] }, ProductPoly { polys: [[0, 0, 0, 0], [225, 25200, 25200, 2822400]] }]
    pub fn generate_sumcheck_proof(&mut self, sum_poly: bit_format::SumPoly<F>, initial_claimed_sum: F, total_bc_bits: u32) -> Proof<F> {
        let mut round_poly = vec![];

        // prover send the initial claimed sum to verifier i.e transcript
        self.transcript.absorb(initial_claimed_sum.into_bigint().to_bytes_be().as_slice());

        // I need to loop tru the sumpoly and product poly arrays and then for each polynomial in them, I partially evaluate to get a univar in b
        let sumpoly_degree = sum_poly.degree();
        let mut current_poly = sum_poly;

        for bit in 0..total_bc_bits {
            let mut reduced_sum_poly_array: Vec<F> = vec![];
            // since the degree of our poly is 2, we need 2 + 1 points to completely desc the poly
            for i in 0..sumpoly_degree + 1 {
                let partially_evaluated_poly = current_poly.partial_evaluate(0, F::from(i as u64)); // i.e keep c (index 0) and eval b over hypercube
    
                // i need to reduce this sum poly which would give me just a vec of F. This is what I would push into te partially_eval_poly_array. Reducing gives an array of evals
                let reduced_sum_poly = partially_evaluated_poly.reduce();
                reduced_sum_poly_array.push(reduced_sum_poly.iter().sum()); // pushing the ys that would be used for univariate interpolation
            }

            // interpolate the reduced_sum_poly_array (ys) and [0, 1, 2] (xs) to get the univariate polynomial
            let univar_poly_fc = univar_poly::Univariatepoly::interpolate(vec![F::from(0), F::from(1), F::from(2)], reduced_sum_poly_array);
            round_poly.push(univar_poly_fc.clone());

            self.transcript.absorb(&univar_poly_fc.coef.iter().map(|y| y.into_bigint().to_bytes_be()).collect::<Vec<_>>().concat());
            let rb = self.transcript.squeeze();
            // evaluate fbc at rb
            current_poly = current_poly.partial_evaluate(0, rb);
        }

        Proof {
            initial_claimed_sum,
            univars_and_sums: round_poly
        } 
    }

}

#[derive(Debug)]
pub struct Proof<F: PrimeField> {
    initial_claimed_sum: F,
    univars_and_sums: Vec<univar_poly::Univariatepoly<F>>
}

struct Verifier<F: PrimeField> {
    polynomial: Vec<F>,
    transcript: bit_format::Transcript<Keccak256, F>,
    proof: Proof<F>
}

impl<F: PrimeField> Verifier<F> {
    fn init(polynomial: Vec<F>, transcript: bit_format::Transcript<Keccak256, F>, proof: Proof<F>) -> Self {
        Self { 
            polynomial,
            transcript,
            proof
        }
    }

    // fn verify(&mut self) -> bool {
    //     let proof = &self.proof.univars_and_sums;
    //     println!("proof length {:?}", proof.len());
    //     println!("proof {:?}", proof);
    //     let mut rand_chal_array: Vec<F> = vec![];
    //     self.transcript.absorb(&self.polynomial.iter().map(|y| y.into_bigint().to_bytes_be()).collect::<Vec<_>>().concat());

    //     let mut claimed_sum = proof[0];
    //     for i in 0..proof.len() {
    //         let (sum, univar_poly) = &proof[i];
    //         // let mut claimed_sum: F = *sum;
    //         println!("univar poly {:?}", univar_poly);
    //         println!("sum {:?}", sum);
    //         let claim: F = univar_poly.iter().sum();
    //         assert_eq!(claimed_sum, claim);
    //         self.transcript.absorb(sum.into_bigint().to_bytes_be().as_slice());
    //         self.transcript.absorb(&univar_poly.iter().map(|y| y.into_bigint().to_bytes_be()).collect::<Vec<_>>().concat());
    //         let rand_chal = self.transcript.squeeze();
    //         rand_chal_array.push(rand_chal);
    //         let current_poly = bit_format::evaluate_interpolate(univar_poly.clone(), 0, rand_chal);
    //         claimed_sum = current_poly[0];

    //         // if i + 1 < proof.len() {
    //         //     assert_eq!(claimed_sum, proof[i + 1].0)
    //         // }

    //         if i == proof.len() - 1 {
    //             // do oracle check
    //             // let mut final_check_sum: Vec<F> = vec![];
    //             let mut poly: Vec<F> = self.polynomial.clone();

    //             println!("rand chal array {:?}", rand_chal_array);
    //             for i in 0..rand_chal_array.len() {
    //                 poly = bit_format::evaluate_interpolate(poly.clone(), 0, rand_chal_array[i]);
    //             }
    //             println!("final sum {:?}", poly[0]);
    //             println!("final pol {:?}", current_poly[0]);
    //             assert_eq!(poly[0], current_poly[0])
    //         }
    //     }
    //     true
    // }

}

#[cfg(test)]
mod test {
    use super::{Prover, Verifier, bit_format::Transcript, Keccak256};
    use sha3::Digest;
    use ark_bn254::Fq;

    // #[test]
    // fn test_transcript() {
    //     let mut transcript = bit_format::Transcript::<Keccak256, Fq>::new(Keccak256::new());

    //     transcript.absorb(b"[0,1]");
    //     transcript.absorb(b"50");

    //     let random_chal = transcript.squeeze();
    //     println!("{}", random_chal);
    // }

    // #[test]
    // fn test_generate_proof() {
    //     let transcript = Transcript::<Keccak256, Fq>::new(Keccak256::new());
    //     let evals = vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(2), Fq::from(4)];
    //     let mut prover = Prover::init(evals.clone(), transcript);
    //     let proof = prover.generate_proof(evals);

    //     println!("{:?}", proof);
    // }

    // #[test]
    // fn test_verifier() {
    //     let evals = vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(2), Fq::from(4)];
    //     let transcript = Transcript::<Keccak256, Fq>::new(Keccak256::new());
    //     let mut prover = Prover::init(evals.clone(), transcript);
    //     let proof = prover.generate_proof(evals.clone());

    //     let mut verifier = Verifier::init(evals.clone(), Transcript::<Keccak256, Fq>::new(Keccak256::new()), proof);
    //     let result = verifier.verify();
    //     assert!(result, "{}", true);
    // }
}