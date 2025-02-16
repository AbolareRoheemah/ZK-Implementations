use ark_ff::PrimeField;
// use multilinear_polynomial::evaluate_interpolate;
pub mod fiat_shamir;
pub mod take_sum_fiat_shamir;
use crate::fiat_shamir::eval_interpol;
// use rand::Rng;

fn main() {
    println!("Hello, world!");
}

// we have a prover and a verifier
// prover needs to:
// 1. Evaluate a polynomial over the boolean hypercube and sum the result
// 2. Partially evaluate a polynomial at a particular var over the hypercube
// 3. Partially evaluate at one var only
// Prover always sends a polynomial and a claimed sum
struct Prover<F: PrimeField> {
    polynomial: Vec<F>,
    // claimed_sum: F
}

impl <F: PrimeField> Prover <F> {
    fn init(polynomial: Vec<F>) -> Self {
        Self { 
            polynomial,
            // claimed_sum
        }
    }

    fn generate_sum_and_univariate_poly(&mut self, evals: &[F]) -> (F, Vec<F>) {
        let claimed_sum: F = evals.iter().sum();
        let evals_half_len = evals.len() / 2;
        let univar_poly = vec![evals.iter().take(evals_half_len).sum(), evals.iter().skip(evals_half_len).sum()];
        (claimed_sum, univar_poly)
    }

    fn proof_func(&mut self, polynomial: Vec<F>, rand_chal: F) -> Vec<F> {
        let poly = eval_interpol::evaluate_interpolate(polynomial, 0, rand_chal);
        poly
    }
}

// Verifier needs to:
// 1. fully evaluate a polynomial at given points
// 2. compare evaluation result with claimed sum
// 3. generate and send random numbers to prover
struct Verifier<F: PrimeField> {
    polynomial: Vec<F>
}

impl <F: PrimeField>Verifier<F> {
    fn init(polynomial: Vec<F>) -> Self {
        Self { polynomial }
    }

    fn verify(&mut self, claimed_sum: F, univar_poly: Vec<F>) -> F {
        // check that the evaluation of the polynomial is equal to claimed sum
        assert_eq!(univar_poly[0] + univar_poly[1], claimed_sum);
        let mut rng = rand::thread_rng();
        let rand_chal = F::rand(&mut rng);
        // self.rand_chal.push(rand_chal);
        rand_chal
    }

    fn oracle_check(&self, univar_poly: Vec<F>, rand_chal_array: Vec<F>) -> bool {
        let mut initial_poly: Vec<F> = self.polynomial.clone();
        let final_univar_poly = eval_interpol::evaluate_interpolate(univar_poly.clone(), 0, rand_chal_array[rand_chal_array.len() - 1]);
        println!("final rand chal {:?}", rand_chal_array[rand_chal_array.len() - 1]);
        for i in 0..rand_chal_array.len() {
            initial_poly = eval_interpol::evaluate_interpolate(initial_poly.clone(), 0, rand_chal_array[i]);
        }
        assert_eq!(initial_poly[0], final_univar_poly[0]);
        true
    }
}

#[cfg(test)]
mod test {
    use std::vec;

    use crate::{Prover, Verifier};
    use ark_bn254::Fq;

    #[test]
    fn test_sum_check_protocol() {
        let mut polynomial = vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(2), Fq::from(4)];
        let mut prover = Prover::init(polynomial.clone());
        let mut verifier = Verifier::init(polynomial.clone());
        let mut rand_chal_array = vec![];
        let no_of_vars = (polynomial.len() as f64).log2() as usize;
        let mut okay = false;
        for i in 0..no_of_vars {
            let (claimed_sum, univar_poly) = prover.generate_sum_and_univariate_poly(&polynomial);
            let new_rand_chal = verifier.verify(claimed_sum, univar_poly.clone());
            rand_chal_array.push(new_rand_chal);
            if i != no_of_vars - 1 {
                let new_poly = prover.proof_func(polynomial.clone(), new_rand_chal);
                // claimed_sum = new_sum;
                polynomial = new_poly;
                // univar_poly = new_poly;56
            } else {
                okay = verifier.oracle_check(univar_poly.clone(), rand_chal_array.clone());
            }
        }
        assert_eq!(okay, true);
    }
}
