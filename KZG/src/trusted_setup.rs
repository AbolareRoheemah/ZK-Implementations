use ark_ec::{AffineRepr, PrimeGroup, ScalarMul};
use ark_bls12_381::{G1Projective as G1, Fr};
use crate::bit_format::get_hypercube;

pub struct Setup {
    pub powers_of_tau: Vec<G1>,
} // Added a comma to properly terminate the struct field

impl Setup {
    pub fn initialize_setup(tau_values: Vec<Fr>) -> Self {
        let check_arrays = get_check_arrays(tau_values);
        let powers_arr = product_array(check_arrays);
        let g1_generator = G1::generator();
        // let enc_taus = g1_generator.batch_mul(&tau_values);
        let generated_elements: Vec<G1> = g1_generator.batch_mul(&powers_arr).into_iter().map(|x| x.into_group()).collect();
        Self { powers_of_tau: generated_elements }
    }
}

fn get_check_arrays(var_arr: Vec<Fr>) -> Vec<Vec<Fr>> {
    let no_of_vars = var_arr.len() as u32;
    let mut collective_check_arr = vec![];
    for i in 0..var_arr.len() {
        let pick_range = 2_u32.pow(no_of_vars - 1 - i as u32);
        let bool_hypercube = get_hypercube(no_of_vars); // = [000, 001, 010, 011, 100, 101, 110, 111]
        let check_0 = Fr::from(1)-var_arr[i];
        let check_1 = var_arr[i];
        let mut indiv_check_arr = vec![];

        let mut count = 0;
        while count < bool_hypercube.len() {
            for _ in 0..pick_range {
                if count < bool_hypercube.len() {
                    indiv_check_arr.push(check_0);
                }
            }
            for _ in 0..pick_range {
                if count < bool_hypercube.len() {
                    indiv_check_arr.push(check_1);
                }
            }
            count += pick_range as usize * 2;
        }
        collective_check_arr.push(indiv_check_arr);
    }
    collective_check_arr
}

fn product_array(collective_check_arr: Vec<Vec<Fr>>) -> Vec<Fr> {
    let mut product_arr = vec![];
    for i in 0..collective_check_arr[0].len() { // 0, 1, 2, 3, 4, 5, 6, 7
        let mut product = Fr::from(0); // 0
        for j in 0..collective_check_arr.len() { //0, 1, 2
            product *= collective_check_arr[j][i]; // product = 0 * 
        }
        product_arr.push(product);
    }
    product_arr
}