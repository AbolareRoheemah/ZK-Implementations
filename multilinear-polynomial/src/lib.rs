use ark_ff::PrimeField;


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

pub fn interpolate_then_evaluate_at_once<F: PrimeField>(no_of_vars: u32, evals: Vec<F>, var_index: usize, var_eval_at: F) -> Vec<F> {
    // panic if the user wants to evaluate at  an inexistent index
    if var_index as u32 >= no_of_vars {
        panic!("You cant evaluate at an inexistent index")
    }
    // pair first
    let pairs = pair_values(no_of_vars, evals, var_index);
    // now i have my y values and I can now use the formular: f(r) = y1 + r(y2 - y1) this would also be in an array
    pairs.iter().map(|(y1, y2)| *y1 + var_eval_at * (*y2 - y1)).collect()
}

// let the hypercube = [000, 001, 010, 011, 100, 101, 110, 111]
fn pair_values<F: PrimeField>(no_of_vars: u32, evals: Vec<F>, var_index: usize) -> Vec<(F, F)> {
    let reps = 2_u32.pow(no_of_vars); // for 3vars = 8
    // panic if array length doesnt match the no of vars inputed
    if evals.len() != reps as usize { // i.e length != 8
        panic!("Wrong length of array input")
    }
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
            if (i + j as usize) < bool_hypercube.len() {
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
