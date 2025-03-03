use ark_ff::PrimeField;
use crate::bit_format::{get_binary_value_as_string, pair_index, interpolate_then_evaluate_at_once};
use crate::fiat_shamir_transcript::{Transcript, conv_poly_to_byte};
use crate::combined_polys::{ProductPoly, SumPoly};


// The goal is to write a rust implementation of a GKR circuit
// So the circuit is a connection of several gates which perform addition or multiplication operations.
// This circuit will be a fan-in 2 circuit which means it takes only 2 inputs and returns one output. 
// The circuit is to be designed in a way that its gates are arranged into layers and the last layer contains just one gate and produces one single result.
// The strategy is to get the values to the gates and then call evaluate on the gates and pass the result to the layer?
// or just leave their results in the gate and form the layer.
// If i pass in the concrete values into the circuit as an array, they will get assigned to the gates based on index

// input layout: The wisdom behind this is so that things can be a bit automatic. Users can just bring their layout and based on that, the circuit will be executed instead of just stringing the gates and passing them into a layer one by one manually.
// I can decide that my layout is this:
// let layout = [
//     2, // Number of layers
//     [ 
//     [[0, 0, 0, 1, 0], [1, 0, 1, 2, 0]],
//     [[0, 1, 0, 1, 0]]
//     ], 
//     [ layer index, ([gate_index, op, (inputs), output],  [gate_index, op, (inputs), output])]
// ];
// struct Layout<F: PrimeField> {
//     no_of_layers: F,
//     layer_def: Vec<Vec<Vec<F>>>,
// }

// The gates: the lowermost layer
// the inpputs would be indexes. Same as the output. This is so that we can arrange the output and easily pick the right inputs.
#[derive(Debug, Clone)]
struct Gate {
    left_input: usize,
    right_input: usize,
    operation: Operation,
    output: usize
}

impl Gate {
    fn new(left_input: usize, right_input: usize, operation: Operation, output: usize) -> Self {
        Self {
            left_input,
            right_input,
            operation,
            output
        }
    }

    fn execute_gate<F: PrimeField>(&mut self, left_input: F, right_input: F) -> F {
        match self.operation {
            Operation::Add => left_input + right_input,
            Operation::Multiply => left_input * right_input
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
enum Operation {
    Add,
    Multiply
}

// The layer: A dynamic collection of gates
#[derive(Clone, Debug)]
pub struct Layer<F: PrimeField> {
    layer_gates: Vec<Gate>,
    pub layer_output: Vec<F>
}

// The circuit: A collection of several layers depending on the input and the operations needed
#[derive(Clone, Debug)]
pub struct Circuit<F: PrimeField> {
    // inputs: Vec<F>,
    // layers: Vec<Vec<F>>,
    pub layers: Vec<Layer<F>>,
    layer_def: Vec<Vec<Vec<usize>>>,
    // ouput: Vec<F>
}

impl <F: PrimeField>Circuit<F> {
   pub fn new(layer_def: Vec<Vec<Vec<usize>>>) -> Self {
        Self {
            // inputs,
            layers: vec![],
            layer_def
        }
    }

    pub fn execute(&mut self, inputs: Vec<F>) -> &Vec<Layer<F>> {
        let no_of_layers = self.layer_def.len();
        let mut mut_inputs = inputs;
        for i in 0..no_of_layers { // for each layer in the layout i.e [[0, 0, 0, 1, 0], [1, 0, 1, 2, 1]] and [[0, 1, 0, 1, 0]] this will run twice
            let layer_length = if i + 1 < no_of_layers { 
                self.layer_def[i].len()
            } else {
                2
            };
            let mut layer = vec![F::zero(); layer_length];
            let mut layer_gate = vec![];
            for gate in &self.layer_def[i] { // for each gate in each layer i.e [0, 0, 0, 1, 0] and [1, 0, 1, 2, 1]
                // let gate_index = gate[0];
                let op = if gate[1] == 0 {
                    Operation::Add
                } else {
                    Operation::Multiply
                };
                let left_input = gate[2];
                let right_input = gate[3];
                let output_index = gate[4];
                let mut new_gate = Gate::new(left_input, right_input, op, output_index);
                let gate_result = new_gate.execute_gate(mut_inputs[left_input], mut_inputs[right_input]);
                layer[output_index] = gate_result;
                layer_gate.push(new_gate);
            }
            mut_inputs = layer.clone();
            let new_layer = Layer {
                layer_gates: layer_gate,
                layer_output: layer
            };
            self.layers.push(new_layer);
            // no_of_layers -= 1;
        }
        self.layers.reverse();
        &self.layers
    }

    pub fn get_bit_len(&self, layer_index: usize) -> (u32, u32) {
        println!("circuit  {:?}", self);
        // let target_layer = &self.layers[layer_index];
        let target_layer = if layer_index < self.layers.len() {
            &self.layers[layer_index]

        } else {
            &self.layers[self.layers.len() - 1]
        };
        let num_gates = target_layer.layer_gates.len();
        println!("num_gates{:?}", num_gates);

        
        // // number of boolean variables needed (log2 of num_gates)
        // // if I have 4 gate on the layer, the output gate i.e the gate at the layer in question is going to be 2 bits i.e output_bit while the input gates will have 3 bits i.e 1 + output gate bits. And since we have 2 input gates then it would be (2 + 1) * 2 + 2 = 8 bits.
        // // so for the layer with 4 gates, output_bit = 2 and therefore, arg = ((2 +1) * 2) + 2 = 8
        // // and for the layer with 2 gates, output_bit = 1 and therefore, arg = ((1 +1) * 2) + 1 = 5
        // // and for the layer with 1 gate, output_bit = 0 and therefore, arg = ((0 +1) * 2) + 1 = 3
        let log_base_2 = (num_gates as f64).log2().ceil() as u32;
        let output_bit = if log_base_2 == 0 {
            log_base_2 + 1
        } else {
            log_base_2
        };
        let total_bc_bit = (log_base_2 + 1) * 2;
        (output_bit, total_bc_bit)
    }
    fn calculate_num_vars_after_blowup(&self, layer_index: usize) -> u32 {
        let target_layer = if layer_index < self.layers.len() {
            &self.layers[layer_index]
        } else {
            &self.layers[self.layers.len() - 1]
        };
        
        let num_gates = target_layer.layer_gates.len(); // Get the number of gates in the target layer

        // Add 1 for the blowup variable
        let base_vars = if num_gates <= 1 {
            1
        } else {
            (num_gates as f64).log2().ceil() as u32
        };
        
        base_vars + 1
    }

    pub fn addi_muli_function(&mut self, layer_index: usize) -> (Vec<F>, Vec<F>) {
        let target_layer = &self.layers[
            if layer_index < self.layers.len() {
                layer_index
            } else {
                self.layers.len() - 1
            }
        ];
        
        let (output_bit, input_bit) = self.get_bit_len(layer_index);
        let bool_hypercube_len = 2_usize.pow(output_bit + input_bit);
        
        // The result will be the sum of all matching gates
        let mut addi_result = vec![F::zero(); bool_hypercube_len];
        let mut muli_result = vec![F::zero(); bool_hypercube_len];
        for gate in target_layer.layer_gates.clone() {
            let gate_op = gate.operation;
            let left_input_index = gate.left_input;
            let right_input_index = gate.right_input;
            let output_index = gate.output;
            let input_bit = output_bit + 1;

            let concat_bits = format!("{}{}{}", get_binary_value_as_string(output_index as u32, output_bit), get_binary_value_as_string(left_input_index as u32, input_bit), get_binary_value_as_string(right_input_index as u32, input_bit));

            let decimal_value = u32::from_str_radix(&concat_bits, 2);
            match decimal_value {
                Ok(value) => if gate_op == Operation::Add {
                    addi_result[value as usize] = F::one();
                } else if gate_op == Operation::Multiply {
                    muli_result[value as usize] = F::one();
                },
                Err(_) => panic!("Error converting binary to decimal")
            };
        };
        (addi_result, muli_result)
    }

    pub fn get_wi_x_func(&self, layer_index: usize, index_of_new_var:u32) -> Vec<F> {
        let target_layer = &self.layers[if layer_index < self.layers.len() {
            layer_index
        } else {
            self.layers.len() - 1
        }
        ];
        let num_vars_after_blowup = self.calculate_num_vars_after_blowup(layer_index);
        // let num_gates = target_layer.layer_gates.len();
        // wi = [3, 5]
        let hypercube_len = 2_u32.pow(num_vars_after_blowup); // 2^1 = 2

        let pairing = pair_index(num_vars_after_blowup, index_of_new_var as usize); // [0, 0, 1, 1, 2, 2, 3, 3]

        let mut resulting_wi_poly = vec![F::zero(); hypercube_len as usize]; //

        // [3,5] =>  [3,3,5,5] for b
        pairing.iter().enumerate().for_each(|(index, (left, right))| {
            if index < target_layer.layer_output.len() {
                resulting_wi_poly[*left as usize] = target_layer.layer_output[index];
                resulting_wi_poly[*right as usize] = target_layer.layer_output[index];
            }
        });

        resulting_wi_poly
    }

    pub fn get_last_layer_wi_x_func(&self, layer_index: usize, index_of_new_var:u32) -> Vec<F> {
        let num_vars_after_blowup = self.calculate_num_vars_after_blowup(layer_index);
        let target_layer = &self.layers[layer_index];
        // let num_gates = target_layer.layer_gates.len();
        // wi = [3, 5]
        let hypercube_len = 2_u32.pow(num_vars_after_blowup); // 2^1 = 2

        let pairing = pair_index(num_vars_after_blowup, index_of_new_var as usize); // [0, 0, 1, 1, 2, 2, 3, 3]

        let mut resulting_wi_poly = vec![F::zero(); hypercube_len as usize]; //

        // [3,5] =>  [3,3,5,5] for b
        pairing.iter().enumerate().for_each(|(index, (left, right))| {
            resulting_wi_poly[*left as usize] = target_layer.layer_output[index];
            resulting_wi_poly[*right as usize] = target_layer.layer_output[index];
        });

        resulting_wi_poly
    }

   pub fn add_wi_x_poly(&self, layer_index: usize) -> Vec<F> {
        // the wi array can either be wb or wc or whatever you call it. but because we want to add or multiply while representing it as 2 different variables, we need to disguise the wi array as though it has that other var but with coef = 0. This is because, you can only add or multiply these polynomials element-wise only if the polys have the same vars and the same no of vars. So at first, we take the wi poly to be wb and so we want to add 0c to it which means the poly now has 2 vars and b is the var at index 0 while c is at index 1. which is why the function takes 1 for wb and 0 for wc. Next we take the wi poly to be wc and we want to add 0b but this time b is at index 0 and c is at index 1. This way, the first array is wbc with an imaginary c and the second is wbc with an imaginary b. Since the two arrays now have the same no of vars and the same vars, we can add or multiply them element wise.
        let wb = self.get_wi_x_func(layer_index, 1); // for wb it is 1
        let wc = self.get_wi_x_func(layer_index, 0); // for wc it is 0
        wb.iter().zip(wc.iter()).map(|(wb, wi)| *wb + *wi).collect()
    }

    pub fn mul_wi_x_poly(&self, layer_index: usize) -> Vec<F> {
        let wb = self.get_wi_x_func(layer_index, 1);
        let wc = self.get_wi_x_func(layer_index, 0);
        wb.iter().zip(wc.iter()).map(|(wb, wi)| *wb * *wi).collect()
    }

    pub fn get_fbc_poly(&mut self, layer_index: usize, a_value: Vec<F>) -> SumPoly<F> {
        let (a_bits, _) = self.get_bit_len(layer_index);
        let (addi_poly, muli_poly) = self.addi_muli_function(layer_index);
        let mut add_rbc = addi_poly;
        let mut mul_rbc = muli_poly;
        let wbc_from_add= self.add_wi_x_poly(layer_index + 1);
        let wbc_from_mul= self.mul_wi_x_poly(layer_index + 1);
        for bit in 0..a_bits {
            add_rbc = interpolate_then_evaluate_at_once(add_rbc, 0, a_value[bit as usize]);
            mul_rbc = interpolate_then_evaluate_at_once(mul_rbc, 0, a_value[bit as usize]);
        }
        let fbc_poly = vec![
            ProductPoly::new(vec![add_rbc, wbc_from_add]),
            ProductPoly::new(vec![mul_rbc, wbc_from_mul])
        ];
        SumPoly {
            product_polys: fbc_poly
        }
    }

    pub fn get_alpha_beta_poly_and_sum(&mut self, layer_index: usize, addi_poly: Vec<F>, muli_poly: Vec<F>, wb: Vec<F>, wc: Vec<F>, r_values: Vec<F>, transcript: &mut Transcript<sha3::Keccak256, F>) -> (SumPoly<F>, F, Vec<F>, Vec<F>) {
        let (new_addi, new_muli, new_claimed_sum, w_rb, w_rc) = self.new_addi_muli(addi_poly, muli_poly, wb, wc, r_values, transcript);
        
        // let mut mul_rbc = muli_poly;
        let wbc_from_add= self.add_wi_x_poly(layer_index + 1);
        let wbc_from_mul= self.mul_wi_x_poly(layer_index + 1);
        let fbc_poly = vec![
            ProductPoly::new(vec![new_addi, wbc_from_add]),
            ProductPoly::new(vec![new_muli, wbc_from_mul])
        ];
        (
            SumPoly {
                product_polys: fbc_poly
            },
            new_claimed_sum,
            w_rb,
            w_rc
        )
    }

    // New addi+1 = alpha * addi+1(rb, b, c) + beta * addi+1(rc, b, c)
    // where alpha & beta are squeezed from transcript, rb = first half of random chal sent from the sumcheck prover and rc = second half of random chal sent from the sumcheck prover
    fn new_addi_muli(&mut self, addi_poly: Vec<F>, muli_poly: Vec<F>, wb: Vec<F>, wc: Vec<F>, r_values: Vec<F>, transcript: &mut Transcript<sha3::Keccak256, F>) -> (Vec<F>, Vec<F>, F, Vec<F>, Vec<F>) {
        let r_len = r_values.len();
        let (rb, rc) = r_values.split_at(r_len / 2);
        let mut addi_rb = addi_poly.clone();
        let mut addi_rc = addi_poly.clone();
        let mut muli_rb = muli_poly.clone();
        let mut muli_rc = muli_poly.clone();
        let mut w_rb = wb.clone();
        let mut w_rc = wc.clone();

        for i in 0..rb.len() {
            addi_rb = interpolate_then_evaluate_at_once(addi_rb, 0, rb[i]);
            addi_rc = interpolate_then_evaluate_at_once(addi_rc, 0, rc[i]);
            muli_rb = interpolate_then_evaluate_at_once(muli_rb, 0, rb[i]);
            muli_rc = interpolate_then_evaluate_at_once(muli_rc, 0, rc[i]);

            // evaluating wb and wc at r values
            w_rb = interpolate_then_evaluate_at_once(w_rb, 0, rb[i]);
            w_rc = interpolate_then_evaluate_at_once(w_rc, 0, rc[i]);
        }
        let binding = conv_poly_to_byte(&w_rb);
        transcript.absorb(binding.as_slice());
        let alpha = transcript.squeeze();

        let binding = conv_poly_to_byte(&w_rc);
        transcript.absorb(binding.as_slice());
        let beta = transcript.squeeze();
        

        let alpha_addi: Vec<F> = addi_rb.iter().map(|eval| *eval * alpha).collect();
        let beta_addi: Vec<F> = addi_rc.iter().map(|eval| *eval * beta).collect();
        let alpha_muli: Vec<F> = muli_rb.iter().map(|eval| *eval * alpha).collect();
        let beta_muli: Vec<F> = muli_rc.iter().map(|eval| *eval * beta).collect();
        let mut new_addi = vec![F::zero(); beta_addi.len()];
        let mut new_muli = vec![F::zero(); beta_muli.len()];
        for i in 0..beta_addi.len() {
            new_addi[i] = alpha_addi[i] + beta_addi[i];
            new_muli[i] = alpha_muli[i] + beta_muli[i];
        }

        // calculating new claimed sum by doing alpha * B + beta * C
        let new_claimed_sum = alpha * w_rb.iter().sum::<F>() + beta * w_rc.iter().sum::<F>();

        (new_addi, new_muli, new_claimed_sum, w_rb, w_rc)
    }


}

#[cfg(test)]
mod test {
    use crate::Circuit;
    use ark_bn254::Fq;

    // #[test]
    // fn test_circuit_operation() {
    //     // What happens here:
    //     // 1. The user comes with their problem. Lets say the function they want to solve is 2a + bc
    //     // 2. They have to create two multiplication gates and one addition gate. I can do this by accepting in my add_layer/execute function, the layout structure. For this now, I could have something like [[layer0, no_gates, [gate1, ], inputs], layer1]
    //     // to totally define the circuit, I need to specify how many layers I have.
    //     // 1. no of layers
    //     // 2. For each layer, I need to specify the gates present and their inputs as indexes
    //     // e.g in layer 0, we have 2 gates
    //     // Then I need to specify the gate operations and inputs
    //     // e.g gate 0 takes index 0, 1, its operation is mul andf it outputs to index 0, gate 1 takes 2, 3 and its op is mul and it ouputs to 1
    //     // In layer 1, we have one gate
    //     // gate 0 takes index 0, 1 from the previous layer, its operation is add and it outputs to 0.
    //     // I can also check the amount of input provided by checking the amount of gates specified for the first (input) layer.
    //     let layout = vec![
    //         vec![
    //             vec![0, 1, 0, 1, 0], // [gate_index, op, left_input, right_input, output_index]
    //             vec![1, 1, 2, 3, 1]
    //         ],
    //         vec![
    //             vec![0, 0, 0, 1, 0]
    //         ]
    //     ]; 
    //     let layout1 = vec![
    //         vec![
    //             vec![0, 0, 0, 1, 0],
    //             vec![1, 1, 2, 3, 1],
    //             vec![1, 1, 4, 5, 2],
    //             vec![1, 1, 6, 7, 3],
    //         ],
    //         vec![
    //             vec![0, 0, 0, 1, 0],
    //             vec![1, 1, 2, 3, 1],
    //         ],
    //         vec![
    //             vec![0, 0, 0, 1, 0]
    //         ]
    //     ]; 
    //     let mut my_circuit: Circuit<Fq> = Circuit::new(layout);
    //     let mut my_circuit1: Circuit<Fq> = Circuit::new(layout1);
    //     let my_inputs = vec![Fq::from(2), Fq::from(3), Fq::from(1), Fq::from(2)];
    //     let my_inputs1 = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4), Fq::from(5), Fq::from(6), Fq::from(7), Fq::from(8)];
    //     let evaluated_layers = my_circuit.execute(my_inputs);
    //     let evaluated_layers1 = my_circuit1.execute(my_inputs1);
    //     // let result = vec![vec![Fq::from(2)]]
    //     println!("my layers{:?}", evaluated_layers);
    //     println!("my layers one{:?}", evaluated_layers1);

    //     // assert_eq!(evaluated_layers, [])
    // }

    #[test]
    fn test_addi_muli_function() {
        let layout = vec![
            vec![
                vec![0, 0, 0, 1, 0],
                vec![1, 1, 2, 3, 1],
                vec![1, 1, 4, 5, 2],
                vec![1, 1, 6, 7, 3],
            ],
            vec![
                vec![0, 0, 0, 1, 0],
                vec![1, 1, 2, 3, 1],
            ],
            vec![
                vec![0, 0, 0, 1, 0]
            ]
        ]; 
        let mut my_circuit: Circuit<Fq> = Circuit::new(layout);
        let my_inputs = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4), Fq::from(5), Fq::from(6), Fq::from(7), Fq::from(8)];
        my_circuit.execute(my_inputs);
        let result = my_circuit.addi_muli_function(0);
        println!("result for addi and muli {:?}", result);
    }

    #[test]
    fn test_get_wi_x_func() {
        // let wi = vec![Fq::from(3), Fq::from(5)];
        let layout = vec![
            vec![
                vec![0, 0, 0, 1, 0],
                vec![1, 1, 2, 3, 1],
                vec![1, 1, 4, 5, 2],
                vec![1, 1, 6, 7, 3],
            ],
            vec![
                vec![0, 0, 0, 1, 0],
                vec![1, 1, 2, 3, 1],
            ],
            vec![
                vec![0, 0, 0, 1, 0]
            ]
        ]; 
        let mut my_circuit: Circuit<Fq> = Circuit::new(layout);
        let my_inputs = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4), Fq::from(5), Fq::from(6), Fq::from(7), Fq::from(8)];
        my_circuit.execute(my_inputs);
        let result = my_circuit.get_wi_x_func(1, 1); // by design, layer index passed into this function cant be zero coz we are always using the Wi from the i+1 layer so if i = 0, the W we would be using would be W(0 +1) = W1
        let resultc = my_circuit.get_wi_x_func(1, 0);
        println!("result for blowup in b {:?}", result);
        println!("result for blowup in c{:?}", resultc);
    }

    #[test]
    fn test_add_and_mul_wi_x_poly() {
        let layout = vec![
            vec![
                vec![0, 0, 0, 1, 0],
                vec![1, 1, 2, 3, 1],
                vec![1, 1, 4, 5, 2],
                vec![1, 1, 6, 7, 3],
            ],
            vec![
                vec![0, 0, 0, 1, 0],
                vec![1, 1, 2, 3, 1],
            ],
            vec![
                vec![0, 0, 0, 1, 0]
            ]
        ]; 
        let mut my_circuit: Circuit<Fq> = Circuit::new(layout);
        let my_inputs = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4), Fq::from(5), Fq::from(6), Fq::from(7), Fq::from(8)];
        my_circuit.execute(my_inputs);
        let wbc_add = my_circuit.add_wi_x_poly(0); // by design, num of vars after blow up cant be less than 2 coz we always start from W1 which when blown up has 2 vars
        let wbc_mul = my_circuit.mul_wi_x_poly(0);
        println!("result for adding blownup polys{:?}", wbc_add);
        println!("result for muling blownup polys{:?}", wbc_mul);
    }

    #[test]
    fn test_get_fbc_poly() {
        let layout = vec![
            vec![
                vec![0, 0, 0, 1, 0],
                vec![1, 1, 2, 3, 1],
                vec![1, 1, 4, 5, 2],
                vec![1, 1, 6, 7, 3],
            ],
            vec![
                vec![0, 0, 0, 1, 0],
                vec![1, 1, 2, 3, 1],
            ],
            vec![
                vec![0, 0, 0, 1, 0]
            ]
        ]; 
        let mut my_circuit: Circuit<Fq> = Circuit::new(layout);
        let my_inputs = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4), Fq::from(5), Fq::from(6), Fq::from(7), Fq::from(8)];
        my_circuit.execute(my_inputs);
        let fbc_poly = my_circuit.get_fbc_poly(0, vec![Fq::from(7898765)]);
        println!("result for fbc poly {:?}", fbc_poly);
    }
}
