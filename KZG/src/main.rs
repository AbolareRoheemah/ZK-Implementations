use ark_bls12_381::{G1Projective as G1, Fr};
use ark_ec::{AffineRepr, PrimeGroup, ScalarMul};
use ark_ff::PrimeField;
pub mod trusted_setup;
pub mod bit_format;
use crate::bit_format::{pair_index, MultilinearPoly, full_evaluate, interpolate_then_evaluate_at_once};
 
fn main() {
    println!("Hello, world!");
}

struct KZGProver {
    polynomial: MultilinearPoly<Fr>,
    powers_of_tau: Vec<G1>
}

impl KZGProver {
    pub fn init(polynomial: MultilinearPoly<Fr>, powers_of_tau: Vec<G1>) -> Self {
        KZGProver { polynomial, powers_of_tau }
    }

    pub fn commit(&self, taus: Vec<Fr>) -> G1 {
        let setup = trusted_setup::Setup::initialize_setup(taus);
        let powers_of_tau = setup.powers_of_tau;
        dot_prod(self.polynomial.clone(), powers_of_tau)
    }

    pub fn open_poly(&self, evaluation_values: &Vec<Fr>) -> Fr {
        let evaluation = full_evaluate(&evaluation_values.len(), self.polynomial.coef_and_exp.clone(), evaluation_values);
        evaluation
    }

    pub fn generate_proof(&self, v: Fr, evaluation_values: &Vec<Fr>) -> Vec<G1> {
        let poly = &self.polynomial.coef_and_exp;
        if poly.len() != evaluation_values.len() {
            panic!("The length of the polynomial and the evaluation values must be equal");
        }
        // let left_numerator: Vec<Fr> = poly.iter().map(|y| *y - v).collect();
        let mut quotients_arr = vec![];
        let mut remainder = poly.clone(); // Create a longer-lived value for remainder
        for i in 0..evaluation_values.len() {
            remainder = interpolate_then_evaluate_at_once(remainder.to_vec(), 0, evaluation_values[i]);
            let mid = remainder.len() / 2;
            let (f_0, f_1) = remainder.split_at(mid);
            let quotient: Vec<Fr> = f_0.iter().zip(f_1.iter()).map(|(x, y)| y - x).collect();
            let blownup_quotient = blowup(quotient, 0, 3);
            let proof = dot_prod(MultilinearPoly::new(blownup_quotient), self.powers_of_tau.to_vec());
            quotients_arr.push(proof);
        }
        quotients_arr
    }
}

pub fn dot_prod(poly: MultilinearPoly<Fr>, powers_of_tau: Vec<G1>) -> G1 {
    if poly.coef_and_exp.len() != powers_of_tau.len() {
        panic!("The length of the polynomial and the powers of tau must be equal");
    }
    let outcome: Vec<G1> = poly.coef_and_exp.iter().zip(powers_of_tau.iter()).map(|(x, y)| y.mul_bigint(x.into_bigint())).collect();

    outcome.iter().sum()
}

pub fn blowup(poly: Vec<Fr>, index_of_new_var:u32, num_vars_after_blowup: u32) -> Vec<Fr> {
    let hypercube_len = 2_u32.pow(num_vars_after_blowup); // 2^1 = 2

    let pairing = pair_index(num_vars_after_blowup, index_of_new_var as usize); // [0, 0, 1, 1, 2, 2, 3, 3]

    let mut resulting_poly = vec![Fr::from(0); hypercube_len as usize]; //

    // [3,5] =>  [3,3,5,5] for b
    pairing.iter().enumerate().for_each(|(index, (left, right))| {
        if index < poly.len() {
            resulting_poly[*left as usize] = poly[index];
            resulting_poly[*right as usize] = poly[index];
        }
    });

    resulting_poly
}