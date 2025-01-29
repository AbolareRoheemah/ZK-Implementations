// use std::iter::{Product, Sum};
// use std::ops::{Add, Mul};
use ark_ff::PrimeField;
use ark_bn254::Fq;
// mod bit_format;

// Multilinear polynomials are polynomials with all variables having the power of one
// We want to be able to represent them and perform evaluations and interpolations on them

#[derive(Debug, PartialEq, Clone)]
struct MultilinearPoly<F: PrimeField> {
    coef_and_exp: Vec<(&'static [F], F)>
}
impl <F: PrimeField> MultilinearPoly<F> {
    fn new(coef: Vec<(&'static [F], F)>) -> Self {
        MultilinearPoly { coef_and_exp: coef }
    }

    // f(a,b,c) = 2ab + 3bc
    // rep as vec![   ([1,1,0], 2), ([0,1,1], 3)    ]
    // fn evaluate(&self, vars: Vec<F>) -> F {
    //     let mut result = F::zero();
    //     for (exp_arr, coef) in &self.coef_and_exp {
    //         let term_value = vars.iter().zip(exp_arr.iter()).map(|(var, exp)| {
    //             var.pow(*exp)
    //         }).product::<F>();
    //         result += coef * term_value;
    //     }
    //     result
    // }

    // this can be pictured like the univariate evaluation where we would find the evaluation
    //  of the polynomial at given points and the construct the interpolating formular from it
    // so for f(a,b,c) = 2ab + 3bc, evaluating over the boolean hypercube gives all zeros except at 
    // points (0,1,1) = 3, (1,1,0) = 2, (1,1,1) = 5
    // f = 
    // fn interpolate(xs: Vec<F>, ys: Vec<F>) -> Self {
    //     let new_coef = xs.iter().zip(ys.iter()).map(|(exp_arr, coef)| {
    //         exp_arr.iter().map(|exp| Self::basis(exp, exp_arr).scalar_mul(coef))
    //     })
    // }

}

// ---------------------------------------------------------------------------------------------------

// Goal: Partial evaluation of multilinear polynomials
// steps:
// 1. Representation: What we are going to be given is an array of the evaluation
//  of the polynomial over the boolean hypercube. So for a 2 var function, 
// we would have 2^2 i.e 4 enteries in the array:
// [
// 00 - 1
// 01 - 2
// 10 - 3
// 11 - 4
// ]
// and for 3 var functions, we would have 2^3 i.e 8 enteries in the array:
// [
// 000 - 1
// 001 - 2
// 010 - 3
// 011 - 4
// 100 - 5
// 101 - 6
// 110 - 7
// 111 - 8
// ]. Now we need a way to know which element of the array represents evaluation at 000
//  and which was evaluated at 001 etc. 
// - Im thinking of asking the user to pass in the number of vars, then check if the length
// of the array provided is equal to 2^(the number passed in). If it is the next step falls in 
// which is to now use the evaluate-interpolate formular to do the partial evaluation 
// at a specified value for a specified index

// We need to find a way to pair thes points. Pairing them means finding lines that connect 
// i.e in the bit, we need to find where our variable of interest is the only changing bit.
// I can do this in 2 ways:
// 1. pick a string of bits, and identify the bit of interest and change that bit of interest 
// to its opposite i.e either 0 or 1. then check the entire array and find if theres any other 
// combination of bit that equals that new string of bit. when you find it, map them/their values
// together and remove the first element you considered. Do this till you exhaust the array.
// 2. Pick a string of bits and identify the bit(s) that are not being cosidered, 
// check the other elements if any other one has their unwanted bit(s) equal to that, then we pair them.

fn get_binary_value(decimal_value: u32, width: u32) -> u32 {
    let mut bits: String = String::from("");
    let mut value = decimal_value;
    while value > 0 {
        let bit_to_add = (value % 2).to_string();
        bits.push_str(&bit_to_add);
        value /= 2;
    }
    bits = bits.chars().rev().collect::<String>();
    while bits.len() < width as usize {
        bits.insert(0, '0');
    }
    let hold: u32 = bits.parse().expect("error");
    println!("each bit {:?}", hold);
    bits.parse().expect("error")
}

fn get_hypercube(no_of_vars: u32) -> Vec<u32> {
    let representations = 2_u32.pow(no_of_vars);
    let mut hypercube: Vec<u32> = vec![];
    for i in 0..representations {
        hypercube.push(get_binary_value(i, no_of_vars))
    }
    println!("hypercube {:?}", hypercube);
    hypercube
}

pub fn evaluate_interpolate<F: PrimeField>(no_of_vars: u32, evals: Vec<F>, var_index: usize, var_eval_at: F) -> Vec<F> {
    // panic if the user wants to evaluate at  an inexistent index
    if var_index as u32 >= no_of_vars {
        panic!("You cant evaluate at an inexistent index")
    }
    // pair first
    let pairs = pair_values(no_of_vars, evals, var_index);
    // now i have my y values and I can now use the formular: f(r) = y1 + r(y2 - y1) this would also be in an array
    pairs.iter().map(|(y1, y2)| *y1 + var_eval_at * (*y2 - y1)).collect()
}

// i need to generate the pairs. I need the bool hypercube and the evals 
// (evaluations over the boolean hypercube) to do that which I already have. 
// Now I think I should first check the index supplied by the user and based on that, 
// I will loop tru the hypercube array to find which other element is equal to it after changing
// the required index. Can I use bitwise operations for this? 

// let the hypercube = [000, 001, 010, 011, 100, 101, 110, 111]
fn pair_values<F: PrimeField>(no_of_vars: u32, evals: Vec<F>, var_index: usize) -> Vec<(F, F)> {
    // let reps = 2_u32.pow(no_of_vars); // for 3vars = 8
    // // panic if array length doesnt match the no of vars inputed
    // if evals.len() != reps as usize { // i.e length != 8
    //     panic!("Wrong length of array input")
    // }
    let pick_range = 2_u32.pow(no_of_vars - 1 - var_index as u32);
    let bool_hypercube = get_hypercube(no_of_vars); // = [000, 001, 010, 011, 100, 101, 110, 111]
    let mut y1s: Vec<F> = vec![];
    let mut y1s_boolhypercube: Vec<u32> = vec![];
    let mut y2s_boolhypercube: Vec<u32> = vec![];
    let mut y2s: Vec<F> = vec![];
    let mut i = 0;
    let var = bool_hypercube[pick_range as usize];
    println!("var are here {:?}", var);
    while i < bool_hypercube.len() { // (0 < 8)
        for j in 0..pick_range {
            if (i + j as usize) < bool_hypercube.len() && (i + j as usize) < evals.len() {
                y1s.push(evals[i + j as usize]);
                y1s_boolhypercube.push(bool_hypercube[i + j as usize]);
                y2s_boolhypercube.push(bool_hypercube[i + j as usize] | var);
            }
        }
        i += pick_range as usize * 2;
    }
    for y in &y2s_boolhypercube {
        if let Some(index) = bool_hypercube.iter().position(|&x| x == *y ) {
            y2s.push(evals[index])
        }
    }
    println!("y1s are here{:?}", y1s);
    println!("y2s are here{:?}", y2s);
    println!("y1bools are here{:?}", y1s_boolhypercube);
    println!("y2bools are here{:?}", y2s_boolhypercube);
    // Collecting pairs of y1s and y2s
    y1s.iter()
        .zip(y2s.iter())
        .map(|(y1, y2)| (*y1, *y2)) // Dereference to match the expected type
        .collect()
}

fn main() {
    let cube = evaluate_interpolate(3, vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(3), Fq::from(0), Fq::from(0), Fq::from(2), Fq::from(5)], 2, Fq::from(3));
    // let cube = evaluate_interpolate(2, vec![Fq::from(0), Fq::from(2), Fq::from(0), Fq::from(5)], 1, Fq::from(5));
    // let cube = get_binary_value(7, 3);.
    println!("{:?}", cube);
}
