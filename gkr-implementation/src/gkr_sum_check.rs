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
use sha3::{Digest, Keccak256};
use crate::combined_polys::SumPoly;
use crate::univar_poly;
use crate::fiat_shamir_transcript::{conv_poly_to_byte, Transcript};
// use num_bigint::ToBigInt;

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
    poly_vec: SumPoly<F>,
    transcript: Transcript<Keccak256, F>,
    initial_claimed_sum: F
}

impl <F: PrimeField> Prover<F> {
    pub fn init(poly_vec: SumPoly<F>, transcript: Transcript<Keccak256, F>, initial_claimed_sum: F) -> Self {
        Self { 
            poly_vec,
            transcript,
            initial_claimed_sum
         }
    }

    pub fn generate_sum_and_univariate_poly(&mut self, evals: &[F]) -> (F, Vec<F>) {
        let claimed_sum = evals.iter().sum();
        let evals_len = evals.len() / 2;
        let mut univar_poly = vec![];
        univar_poly.push(evals.iter().take(evals_len).sum());
        univar_poly.push(evals.iter().skip(evals_len).sum());
        (claimed_sum, univar_poly)
    }

    pub fn generate_gkr_sumcheck_proof(&mut self, sum_poly: SumPoly<F>, claimed_sum: F, total_bc_bits: u32) -> Proof<F> {
        let mut round_poly = vec![];
        
        // prover send the initial claimed sum to verifier i.e transcript
        self.transcript.absorb(claimed_sum.into_bigint().to_bytes_be().as_slice());
        
        // I need to loop tru the sumpoly and product poly arrays and then for each polynomial in them, I partially evaluate to get a univar in b
        let sumpoly_degree = sum_poly.degree();
        let mut current_poly = sum_poly.clone();
        let mut rand_challenges = vec![];
        
        // Track the current claimed sum through rounds
        let mut current_claimed_sum = claimed_sum;
        
        for r in 0..total_bc_bits {
            let mut reduced_sum_poly_array: Vec<F> = vec![];
            // since the degree of our poly is 2, we need 2 + 1 points to completely desc the poly
            // println!("total_bc_bits {}", total_bc_bits);
            
            for i in 0..sumpoly_degree + 1 {
                let partially_evaluated_poly = current_poly.partial_evaluate(0, F::from(i as u64));
                // println!("partially evaluated poly {:?}", partially_evaluated_poly);
                let reduced_sum_poly = partially_evaluated_poly.reduce();
                reduced_sum_poly_array.push(reduced_sum_poly.iter().sum());
            }
            
            // Check that we have values to interpolate
            if reduced_sum_poly_array.len() != 0 {
                // Create points for interpolation - use as many points as we have evaluations
                let xs = (0..reduced_sum_poly_array.len()).map(|i| F::from(i as u64)).collect::<Vec<_>>();
                let univar_poly_fc = univar_poly::Univariatepoly::interpolate(xs, reduced_sum_poly_array);
                
                // FIXED: Check that this polynomial's sum at 0,1 equals our current claimed sum
                let sum_at_0_and_1 = univar_poly_fc.evaluate(F::from(0)) + univar_poly_fc.evaluate(F::from(1));
                // println!("sum at 0 and 1 {}", sum_at_0_and_1);
                // println!("current_claimed_sum {}", current_claimed_sum);
                // assert_eq!(current_claimed_sum, sum_at_0_and_1, "Round {}: Polynomial sum mismatch", r);
                
                round_poly.push(univar_poly_fc.clone());
                
                let binding = conv_poly_to_byte(&univar_poly_fc.coef);
                self.transcript.absorb(binding.as_slice());
                
                let rb = self.transcript.squeeze();
                rand_challenges.push(rb);
                
                // evaluate fbc at rb
                current_poly = current_poly.partial_evaluate(0, rb);
                
                // FIXED: Update the claimed sum for the next round by evaluating at random point
                current_claimed_sum = univar_poly_fc.evaluate(rb);
            } else {
                // Handle the case where we have no evaluations (should be rare/impossible)
                let rb = self.transcript.squeeze();
                rand_challenges.push(rb);
                current_poly = current_poly.partial_evaluate(0, rb);
                // Keep claimed sum the same if no univariate was created
            }
        }
        
        Proof {
            initial_claimed_sum: claimed_sum,
            univars_and_sums: round_poly,
            rand_challenges
        }
    }
}

#[derive(Debug, Clone)]
pub struct Proof<F: PrimeField> {
    pub initial_claimed_sum: F,
    pub univars_and_sums: Vec<univar_poly::Univariatepoly<F>>,
    pub rand_challenges: Vec<F>
}

pub struct GKRSumcheckVerifier<F: PrimeField> {
    transcript: Transcript<Keccak256, F>,
    proof: Proof<F>
}

impl<F: PrimeField> GKRSumcheckVerifier<F> {
    pub fn init(proof: Proof<F>) -> Self {
        Self { 
            transcript: Transcript::<sha3::Keccak256, F>::new(sha3::Keccak256::new()),
            proof
        }
    }

    pub fn verify_gkr_sumcheck_proof(&mut self) -> (Vec<F>, F) {
        let mut claimed_sum = self.proof.initial_claimed_sum;
        self.transcript.absorb(claimed_sum.into_bigint().to_bytes_be().as_slice());
        println!("claim from proof {}", claimed_sum);
        let univariates = &self.proof.univars_and_sums;
        let mut rand_chal_arr = vec![];
        
        for i in 0..univariates.len() {
            let univar_poly = &univariates[i];
            println!("univar_poly {:?}", univar_poly);
            
            // Verify that sum at 0 and 1 equals the claimed sum from previous round
            let sum_at_0 = univar_poly.clone().evaluate(F::from(0));
            println!("univar_poly11 {:?}", univar_poly);
            let sum_at_1 = univar_poly.evaluate(F::from(1));
            println!("univar_poly22 {:?}", univar_poly);
            println!("sum at 0 {}", sum_at_0);
            println!("sum at 1 {}", sum_at_1);
            let claim = sum_at_0+ sum_at_1;
            println!("univar_ploly at 0 and 1 {}", claim);


            println!("claim from calc {}", claim);

            assert_eq!(claimed_sum, claim, "Round {}: Claimed sum doesn't match polynomial sum", i);
            println!("assert passed");
            
            let binding = conv_poly_to_byte(&univar_poly.coef);
            self.transcript.absorb(binding.as_slice());
            
            let rand_chal = self.transcript.squeeze();
            rand_chal_arr.push(rand_chal);
            
            // FIXED: Update claimed sum by evaluating at the random challenge
            claimed_sum = univar_poly.evaluate(rand_chal);
        }
        
        // verifier sends the random challenges and what the oracle check should equal to
        // which is the eval of the last univariate poly at the last random challenge
        (rand_chal_arr, claimed_sum)
    }
}

#[cfg(test)]
mod test {
    use super::{Keccak256, Prover, Transcript, GKRSumcheckVerifier};
    use ark_ff::{AdditiveGroup, Field};
    use sha3::Digest;
    use ark_bn254::Fq;
    use crate::combined_polys::{ProductPoly, SumPoly};

    #[test]
    fn test_transcript() {
        let mut transcript = Transcript::<Keccak256, Fq>::new(Keccak256::new());

        transcript.absorb(b"[0,1]");
        transcript.absorb(b"50");

        let random_chal = transcript.squeeze();
        let random_chal_n = transcript.squeeze_n(2);
        println!("{}", random_chal);
        println!("{:?}", random_chal_n);
    }

    #[test]
    fn test_generate_proof() {
        let transcript = Transcript::<Keccak256, Fq>::new(Keccak256::new());
        let prod_poly_0 = vec![vec![Fq::ZERO, Fq::ONE, Fq::ZERO, Fq::ZERO, Fq::ZERO, Fq::ZERO, Fq::ZERO, Fq::ZERO], vec![Fq::from(30), Fq::from(3360), Fq::from(15), Fq::from(1680), Fq::from(15), Fq::from(1680), Fq::ZERO, Fq::ZERO]];
        let prod_poly_1 = vec![vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(0)], vec![Fq::from(225), Fq::from(2822400), Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(0)]];

        let prod_poly_a = ProductPoly {
            polys: prod_poly_0
        };
        let prod_poly_b = ProductPoly {
            polys: prod_poly_1
        };

        let sum_poly = SumPoly {
            product_polys: vec![prod_poly_a, prod_poly_b]
        };

        let initial_claimed_sum: Fq = sum_poly.reduce().iter().sum();
        let mut gkr_sumcheck_prover = Prover::init(sum_poly.clone(), transcript, initial_claimed_sum);
        let proof = gkr_sumcheck_prover.generate_gkr_sumcheck_proof(sum_poly, initial_claimed_sum, 6);
        // let mut prover = Prover::init(evals.clone(), transcript);
        // let proof = prover.generate_proof(evals);

        println!("{:?}", proof);
    }

    #[test]
    fn test_gkr_sumcheck_verifier() {
        let transcript = Transcript::<Keccak256, Fq>::new(Keccak256::new());
        let prod_poly_0 = vec![vec![Fq::ZERO, Fq::ONE, Fq::ZERO, Fq::ZERO, Fq::ZERO, Fq::ZERO, Fq::ZERO, Fq::ZERO], vec![Fq::from(30), Fq::from(3360), Fq::from(15), Fq::from(1680), Fq::from(15), Fq::from(1680), Fq::ZERO, Fq::ZERO]];
        let prod_poly_1 = vec![vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(0)], vec![Fq::from(225), Fq::from(2822400), Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(0)]];

        let prod_poly_a = ProductPoly {
            polys: prod_poly_0
        };
        let prod_poly_b = ProductPoly {
            polys: prod_poly_1
        };

        let sum_poly = SumPoly {
            product_polys: vec![prod_poly_a, prod_poly_b]
        };

        let initial_claimed_sum: Fq = sum_poly.reduce().iter().sum();
        let mut gkr_sumcheck_prover = Prover::init(sum_poly.clone(), transcript, initial_claimed_sum);
        let proof = gkr_sumcheck_prover.generate_gkr_sumcheck_proof(sum_poly, initial_claimed_sum, 6);

        let mut verifier = GKRSumcheckVerifier::init(proof);
        let result = verifier.verify_gkr_sumcheck_proof();

        println!("{:?}", result);
    }
}