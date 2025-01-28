use ark_ff::PrimeField;
// use multilinear_polynomial::evaluate_interpolate;
pub mod fiat_shamir;

fn main() {
    println!("Hello, world!");
}

// we have a prover and a verifier
// prover needs to:
// 1. Evaluate a polynomial over the boolean hypercube and sum the result
// 2. Partially evaluate a polynomial at a particular var over the hypercube
// 3. Partially evaluate at one var only
// Prover always sends a polynomial and a claimed sum
struct Prover<F: PrimeField>  {
    polynomial: Vec<F>
}

// Verifier needs to:
// 1. fully evaluate a polynomial at given points
// 2. compare evaluation result with claimed sum
// 3. generate and send random numbers to prover
struct Verifier {

}

impl <F: PrimeField> Prover <F> {

}

