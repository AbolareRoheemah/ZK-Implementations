use core::num;
use std::ops::Add;
use ark_ff::{BigInteger, PrimeField};
pub mod bit_format;
pub mod univar_poly;
pub mod gkr_sum_check;
use sha3::{Digest, Keccak256};
fn main() {
    println!("Hello, world!");
}
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
struct Layer<F: PrimeField> {
    layer_gates: Vec<Gate>,
    layer_output: Vec<F>
}

// The circuit: A collection of several layers depending on the input and the operations needed
#[derive(Clone, Debug)]
struct Circuit<F: PrimeField> {
    // inputs: Vec<F>,
    // layers: Vec<Vec<F>>,
    layers: Vec<Layer<F>>,
    layer_def: Vec<Vec<Vec<usize>>>,
    // ouput: Vec<F>
}

impl <F: PrimeField>Circuit<F> {
    fn new(layer_def: Vec<Vec<Vec<usize>>>) -> Self {
        Self {
            // inputs,
            layers: vec![],
            layer_def
        }
    }

    fn execute(&mut self, inputs: Vec<F>) -> &Vec<Layer<F>> {
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

    fn get_bit_len(&self, layer_index: usize) -> (u32, u32) {
        let target_layer = &self.layers[layer_index];
        let num_gates = target_layer.layer_gates.len();
        
        // number of boolean variables needed (log2 of num_gates)
        // if I have 4 gate on the layer, the output gate i.e the gate at the layer in question is going to be 2 bits i.e output_bit while the input gates will have 3 bits i.e 1 + output gate bits. And since we have 2 input gates then it would be (2 + 1) * 2 + 2 = 8 bits.
        // so for the layer with 4 gates, output_bit = 2 and therefore, arg = ((2 +1) * 2) + 2 = 8
        // and for the layer with 2 gates, output_bit = 1 and therefore, arg = ((1 +1) * 2) + 1 = 5
        // and for the layer with 1 gate, output_bit = 0 and therefore, arg = ((0 +1) * 2) + 1 = 3
        let output_bit = if (num_gates as f64).log2().ceil() as u32 == 0 {
            (num_gates as f64).log2().ceil() as u32 + 1
        } else {
            (num_gates as f64).log2().ceil() as u32
        };
        (output_bit, (output_bit + 1) * 2)
    }

    fn calculate_num_vars_after_blowup(&self, layer_index: usize) -> u32 {
        let target_layer = &self.layers[layer_index + 1]; // We use i+1 layer
        let num_gates = target_layer.layer_gates.len();
        
        // Calculate number of variables needed to represent indices (log2(num_gates))
        // Add 1 for the blowup variable
        let base_vars = if num_gates <= 1 {
            1
        } else {
            (num_gates as f64).log2().ceil() as u32
        };
        
        base_vars + 1
    }

    fn addi_muli_function(&mut self, layer_index: usize) -> (Vec<F>, Vec<F>) {
        let target_layer = &self.layers[layer_index];
        
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

            let concat_bits = format!("{}{}{}", bit_format::get_binary_value_as_string(output_index as u32, output_bit), bit_format::get_binary_value_as_string(left_input_index as u32, input_bit), bit_format::get_binary_value_as_string(right_input_index as u32, input_bit));

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
        // println!("addi_result{:?}", addi_result);
        (addi_result, muli_result)
    }

    pub fn get_wi_x_func(&self, layer_index: usize, index_of_new_var:u32) -> Vec<F> {
        let num_vars_after_blowup = self.calculate_num_vars_after_blowup(layer_index);
        let target_layer = &self.layers[layer_index];
        let num_gates = target_layer.layer_gates.len();
        // wi = [3, 5]
        let hypercube_len = 2_u32.pow(num_vars_after_blowup); // 2^1 = 2

        let pairing = bit_format::pair_index(num_vars_after_blowup, index_of_new_var as usize); // [0, 0, 1, 1, 2, 2, 3, 3]
        println!("pairing {:?}", pairing);

        let mut resulting_wi_poly = vec![F::zero(); hypercube_len as usize]; //

        // [3,5] =>  [3,3,5,5] for b
        pairing.iter().enumerate().for_each(|(index, (left, right))| {
            resulting_wi_poly[*left as usize] = target_layer.layer_output[index];
            resulting_wi_poly[*right as usize] = target_layer.layer_output[index];
        });

        resulting_wi_poly
    }

    fn add_wi_x_poly(&self, layer_index: usize) -> Vec<F> {
        let wb = self.get_wi_x_func(layer_index, 1); // for wb it is 1
        let wc = self.get_wi_x_func(layer_index, 0); // for wc it is 0
        wb.iter().zip(wc.iter()).map(|(wb, wi)| *wb + *wi).collect()
    }

    fn mul_wi_x_poly(&self, layer_index: usize) -> Vec<F> {
        let wb = self.get_wi_x_func(layer_index, 1);
        let wc = self.get_wi_x_func(layer_index, 0);
        wb.iter().zip(wc.iter()).map(|(wb, wi)| *wb * *wi).collect()
    }

    fn get_fbc_poly(&mut self, layer_index: usize, a_value: Vec<F>) -> Vec<bit_format::ProductPoly<F>> {
        let num_vars_after_blowup = self.calculate_num_vars_after_blowup(layer_index);
        let (a_bits, _) = self.get_bit_len(layer_index);
        let (addi_poly, muli_poly) = self.addi_muli_function(layer_index);
        let mut add_rbc = addi_poly;
        let mut mul_rbc = muli_poly;
        let wbc_from_add= self.add_wi_x_poly(layer_index + 1);
        let wbc_from_mul= self.mul_wi_x_poly(layer_index + 1);
        for bit in 0..a_bits {
            add_rbc = bit_format::evaluate_interpolate(add_rbc, 0, a_value[bit as usize]);
            mul_rbc = bit_format::evaluate_interpolate(mul_rbc, 0, a_value[bit as usize]);
        }
        let fbc_poly = vec![
            bit_format::ProductPoly::new(vec![add_rbc, wbc_from_add]),
            bit_format::ProductPoly::new(vec![mul_rbc, wbc_from_mul])
        ];
        fbc_poly
    }
}

struct GKRProver<F: PrimeField>  {
    transcript: bit_format::Transcript<sha3::Keccak256, F>,
    w_poly: Vec<F>,
    circuit: Circuit<F>
}

impl <F: PrimeField>GKRProver<F> {
    fn init(transcript: bit_format::Transcript<sha3::Keccak256, F>, w_poly: Vec<F>, circuit: Circuit<F>) -> Self {
        Self {
            transcript,
            w_poly,
            circuit
        }
    }

    fn execute_circuit(&mut self, inputs: Vec<F>) -> &Vec<Layer<F>> {
        self.circuit.execute(inputs)
    }

    fn invoke_sum_check_prover(&mut self) -> Vec<gkr_sum_check::Proof<F>> {
        let mut circuit = self.circuit.clone();
        let executed_layers = circuit.layers.clone();
        let (input_bit, total_bc_bits) = circuit.get_bit_len(0);
        let mut sumcheck_proof = vec![];
        
        for i in 0..input_bit {
            let w_poly = &executed_layers[i as usize].layer_output;
            let mut gkr_transcript = bit_format::Transcript::<sha3::Keccak256, F>::new(sha3::Keccak256::new());
            gkr_transcript.absorb(&w_poly.iter().map(|y| y.into_bigint().to_bytes_be()).collect::<Vec<_>>().concat());
            
            let mut a_values = vec![F::zero(); input_bit as usize];
            let mut initial_claimed_sum = F::zero();
            
            if i == 0 {
                initial_claimed_sum = w_poly[0];
            } else {
                let rand_no = gkr_transcript.squeeze();
                // Set the i-th position in a_values to the random value
                a_values[i as usize - 1] = rand_no;
                // initial_claimed_sum = bit_format::evaluate_interpolate(w_poly.to_vec(), 0, rand_no)[i as usize];
            }

            let initial_fbc_poly = circuit.get_fbc_poly(0, a_values);
            let mut sumcheck_prover = gkr_sum_check::Prover::init(initial_fbc_poly.clone(), gkr_transcript);

            sumcheck_proof.push(sumcheck_prover.generate_sumcheck_proof(
                bit_format::SumPoly { product_polys: initial_fbc_poly },
                initial_claimed_sum,
                total_bc_bits
            ));
        }
        
        sumcheck_proof
    }

}

struct GkrVerifier<F: PrimeField> {
    transcript: bit_format::Transcript<sha3::Keccak256, F>,
}

impl <F: PrimeField> GkrVerifier<F> {
    fn init(transcript: bit_format::Transcript<sha3::Keccak256, F>) -> Self {
        Self {
            transcript
        }
    }

    fn verify_gkr_prrof(&mut self, proof: Vec<gkr_sum_check::Proof<F>>) {
        
    }
}



#[cfg(test)]
mod test {
    use crate::{bit_format, Circuit};
    use ark_bn254::Fq;
    use ark_ff::PrimeField;

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
        let fbc_poly = my_circuit.get_fbc_poly(0, vec![Fq::from(0)]);
        println!("result for fbc poly{:?}", fbc_poly);
    }
}
