// use std::iter::{Product, Sum};
// use std::ops::{Add, Mul};
use ark_ff::PrimeField;
use ark_bn254::Fq;
use std::marker::PhantomData;
// mod bit_format;

// Multilinear polynomials are polynomials with all variables having the power of one
// We want to be able to represent them and perform evaluations and interpolations on them

#[derive(Debug, PartialEq, Clone)]
pub struct MultilinearPoly<F: PrimeField> {
    pub coef_and_exp: Vec<F>
}
impl <F: PrimeField> MultilinearPoly<F> {
    pub fn new(coef: Vec<F>) -> Self {
        MultilinearPoly { coef_and_exp: coef }
    }
}

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
    // let hold: u32 = bits.parse().expect("error");
    bits.parse().expect("error")
}

pub fn get_binary_value_as_string(decimal_value: u32, width: u32) -> String {
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
    bits
}

pub fn get_hypercube(no_of_vars: u32) -> Vec<u32> {
    let representations = 2_u32.pow(no_of_vars);
    let mut hypercube: Vec<u32> = vec![];
    for i in 0..representations {
        hypercube.push(get_binary_value(i, no_of_vars))
    }
    hypercube
}

pub fn interpolate_then_evaluate_at_once<F: PrimeField>(evals: Vec<F>, var_index: usize, var_eval_at: F) -> Vec<F> {
    // panic if the user wants to evaluate at  an inexistent index
    let no_of_vars = (evals.len() as f64).log2() as u32;
    if var_index as u32 >= no_of_vars {
        panic!("You cant evaluate at an inexistent index")
    }
    // pair first
    let pairs = pair_values(no_of_vars, evals, var_index);
    // now i have my y values and I can now use the formular: f(r) = y1 + r(y2 - y1) this would also be in an array
    pairs.iter().map(|(y1, y2)| *y1 + var_eval_at * (*y2 - y1)).collect()
}

pub fn full_evaluate<F: PrimeField>(no_of_evaluations: &usize, poly: Vec<F>, eval_points: &Vec<F>) -> F {
    let mut result = poly;
    for i in 0..*no_of_evaluations {
        result = interpolate_then_evaluate_at_once(result, 0, eval_points[i])
    }
    result[0]
}

// i need to generate the pairs. I need the bool hypercube and the evals 
// (evaluations over the boolean hypercube) to do that which I already have. 
// Now I think I should first check the index supplied by the user and based on that, 
// I will loop tru the hypercube array to find which other element is equal to it after changing
// the required index. Can I use bitwise operations for this? 

// let the hypercube = [000, 001, 010, 011, 100, 101, 110, 111]
pub fn pair_values<F: PrimeField>(no_of_vars: u32, evals: Vec<F>, var_index: usize) -> Vec<(F, F)> {
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
            if index < evals.len() {
                y2s.push(evals[index])
            }
        }
    }
    // Collecting pairs of y1s and y2s
    y1s.iter()
        .zip(y2s.iter())
        .map(|(y1, y2)| (*y1, *y2)) // Dereference to match the expected type
        .collect()
}

pub fn pair_index(no_of_vars: u32, var_index: usize) -> Vec<(u32, u32)> {
    let pick_range = if no_of_vars != 1 {
        2_u32.pow(no_of_vars - 1 - var_index as u32)
    } else {
        1
    };
    let bool_hypercube = get_hypercube(no_of_vars); // = [000, 001, 010, 011, 100, 101, 110, 111]
    let mut y1s_boolhypercube: Vec<u32> = vec![];
    let mut y2s_boolhypercube: Vec<u32> = vec![];
    let mut i = 0;
    let var = bool_hypercube[pick_range as usize];
    while i < bool_hypercube.len() { // (0 < 8)
        for j in 0..pick_range {
            if (i + j as usize) < bool_hypercube.len() {
                y1s_boolhypercube.push((i + j as usize).try_into().unwrap());
                let pair_index = bool_hypercube.iter().position(|&x| x == bool_hypercube[i + j as usize] | var).unwrap(); 
                y2s_boolhypercube.push(pair_index.try_into().unwrap());
            }
        }
        i += pick_range as usize * 2;
    }
    // Collecting pairs of y1s and y2s
    y1s_boolhypercube.iter()
        .zip(y2s_boolhypercube.iter())
        .map(|(y1, y2)| (*y1, *y2)) // Dereference to match the expected type
        .collect()
}

fn main() {
    let cube = interpolate_then_evaluate_at_once(vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(3), Fq::from(0), Fq::from(0), Fq::from(2), Fq::from(5)], 2, Fq::from(3));
    // let cube = interpolate_then_evaluate_at_once(2, vec![Fq::from(0), Fq::from(2), Fq::from(0), Fq::from(5)], 1, Fq::from(5));
    // let cube = get_binary_value(7, 3);.
    println!("cube {:?}", cube);
}
