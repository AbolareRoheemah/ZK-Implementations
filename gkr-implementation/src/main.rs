use core::num;
use std::ops::Add;
pub mod bit_format;

use ark_ff::PrimeField;
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
#[derive(Debug)]
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
#[derive(Debug)]
enum Operation {
    Add,
    Multiply
}

// The layer: A dynamic collection of gates
#[derive(Debug)]
struct Layer<F: PrimeField> {
    layer_gates: Vec<Gate>,
    layer_output: Vec<F>
}

// The circuit: A collection of several layers depending on the input and the operations needed
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

    // let layout = [
    //     2, // Number of layers
    //     [ 
    //     [[0, 0, 0, 1, 0], [1, 0, 1, 2, 0]],
    //     [[0, 1, 0, 1, 0]]
    //     ], 
    //     [ layer index, ([gate_index, op, (inputs), output],  [gate_index, op, (inputs), output])]
    // ];

    fn execute(&mut self, inputs: Vec<F>) -> &Vec<Layer<F>> {
        // if inputs.len() >= self.layers.len() {
        //     panic!("")
        // }
        let no_of_layers = self.layer_def.len();
        let mut mut_inputs = inputs;
        for i in 0..no_of_layers { // for each layer in the layout i.e [[0, 0, 0, 1, 0], [1, 0, 1, 2, 1]] and [[0, 1, 0, 1, 0]] this will run twice
            let layer_length = if i + 1 < no_of_layers { 
                self.layer_def[i].len()
            } else {
                1
            };
            let mut layer = vec![F::zero(); layer_length];
            let mut layer_gate = vec![];
            for gate in &self.layer_def[i] { // for each gate in each layer i.e [0, 0, 0, 1, 0] and [1, 0, 1, 2, 1]
                let gate_index = gate[0];
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
        &self.layers
    }

    fn addi(&mut self, layer_index: usize, left_input_index: usize, right_input_index: usize, output_index: usize) -> usize {
        let target_layer = &self.layer_def[layer_index];
        for gate in target_layer {
            if gate[2] == left_input_index && gate[3] == right_input_index && gate[4] == output_index && gate[1] == 0 {
                return 1;
            }
        }
        0
    }

    fn muli(&mut self, layer_index: usize, left_input_index: usize, right_input_index: usize, output_index: usize) -> usize {
        let target_layer = &self.layer_def[layer_index];
        for gate in target_layer {
            if gate[2] == left_input_index && gate[3] == right_input_index && gate[4] == output_index && gate[1] == 1 {
                return 1;
            }
        }
        0
    }

    // what i want to do is to loop tru the target layer and use the boolean values to evaluate the addi and muli using check0/check1. So I will check the layer for how many gates it has, if it haas 4 gates, then it means the hypercube is in 2 bits i.e 4/2. If it has 2 gates then the bit is 2/2 = 1 bit and if it has 1 gate then the bit is just 0. I will now pass these boolean values as input to the addi and muli functions. It will loop tru the bool values which are strings and if it is '0', then it will do check0 i.e 1-x where x can be random numbers we want to evaluate at.
    fn compute_selector_term(&self, x: F, b: bool) -> F {
        if b {
            x // If b is 1, return x
        } else {
            F::one() - x // If b is 0, return 1-x
        }
    }
    
    fn compute_selector_product(&self, x_values: &[F], bits: &str) -> F {
        let mut result = F::one();
        for (i, bit) in bits.chars().enumerate() {
            let term = self.compute_selector_term(x_values[i], bit == '1');
            result *= term;
        }
        result
    }

    fn addi_muli_function(&mut self, layer_index: usize) -> Vec<F> {
        let target_layer = &self.layer_def[layer_index];
        let num_gates = target_layer.len();
        
        // number of boolean variables needed (log2 of num_gates)
        let output_bit = (num_gates as f64).log2().ceil() as u32;
        // if I have 4 gate on the layer, the output gate i.e the gate at the layer in question is going to be 2 bits i.e output_bit while the input gates will have 3 bits i.e 1 + output gate bits. And since we have 2 input gates then it would be (2 + 1) * 2 + 2 = 8 bits.
        // so for the layer with 4 gates, output_bit = 2 and therefore, arg = ((2 +1) * 2) + 2 = 8
        // and for the layer with 2 gates, output_bit = 1 and therefore, arg = ((1 +1) * 2) + 1 = 5
        // and for the layer with 1 gate, output_bit = 0 and therefore, arg = ((0 +1) * 2) + 1 = 3
        // this also means that if the output bit is of len 2 i.e output_bit, then the input bits will be of len 3 each i.e output_bit + 1
        let arg = if output_bit != 0 {
            ((output_bit + 1) * 2) + output_bit
        } else {
            ((output_bit + 1) * 2) + 1
        };
        let bool_hypercube_len = 2_usize.pow(arg);
        
        // The result will be the sum of all matching gates
        let mut result = vec![F::zero(); bool_hypercube_len];
        for gate in target_layer {
            let left_input_index = gate[2];
            let right_input_index = gate[3];
            let output_index = gate[4];
            let input_bit = output_bit + 1;

            let concat_bits = format!("{}{}{}", get_binary_value(output_index as u32, output_bit), get_binary_value(left_input_index as u32, input_bit), get_binary_value(right_input_index as u32, input_bit));

            let decimal_value = u32::from_str_radix(&concat_bits, 2);
            match decimal_value {
                Ok(value) => result[value as usize] = F::one(),
                Err(_) => panic!("Error converting binary to decimal")
            };
        };
        println!("result{:?}", result);
        result
    }
}

fn get_binary_value(decimal_value: u32, width: u32) -> String {
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

#[cfg(test)]
mod test {
    use crate::Circuit;
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

    // #[test]
    // fn test_addi() {
    //     let layout = vec![
    //         vec![
    //             vec![0, 1, 0, 1, 0], // [gate_index, op, left_input, right_input, output_index]
    //             vec![1, 1, 2, 3, 1]
    //         ],
    //         vec![
    //             vec![0, 0, 0, 1, 0]
    //         ]
    //     ]; 
    //     let mut my_circuit: Circuit<Fq> = Circuit::new(layout);
    //     let result = my_circuit.addi(0, 0, 1, 0); // check for an addition gate in layer 0 that takes 0, 1 and outputs to 0
    //     let result1 = my_circuit.addi(1, 0, 1, 0); // check for an addition gate in layer 1 that takes 0, 1 and outputs to 0
    //     assert_eq!(result, 0);
    //     assert_eq!(result1, 1);
    // }

    // #[test]
    // fn test_muli() {
    //     let layout = vec![
    //         vec![
    //             vec![0, 1, 0, 1, 0], // [gate_index, op, left_input, right_input, output_index]
    //             vec![1, 1, 2, 3, 1]
    //         ],
    //         vec![
    //             vec![0, 0, 0, 1, 0]
    //         ]
    //     ]; 
    //     let mut my_circuit: Circuit<Fq> = Circuit::new(layout);
    //     let result = my_circuit.muli(0, 0, 1, 0); // check for an multiplication gate in layer 0 that takes 0, 1 and outputs to 0
    //     let result2 = my_circuit.muli(0, 2, 3, 1); // check for an multiplication gate in layer 0 that takes 0, 1 and outputs to 0
    //     let result1 = my_circuit.muli(1, 0, 1, 0); // check for an multiplication gate in layer 1 that takes 0, 1 and outputs to 0
    //     assert_eq!(result, 1);
    //     assert_eq!(result1, 0);
    //     assert_eq!(result2, 1);
    // }

    #[test]
    fn test_addi_muli_function() {
        let layout = vec![
            vec![
                vec![0, 1, 0, 1, 0], // [gate_index, op, left_input, right_input, output_index]
                vec![1, 1, 2, 3, 1]
            ],
            vec![
                vec![0, 0, 0, 1, 0]
            ]
        ]; 
        let mut my_circuit: Circuit<Fq> = Circuit::new(layout);
        // let my_inputs = vec![Fq::from(2), Fq::from(3), Fq::from(1), Fq::from(2)];
        // let result = my_circuit.addi_or_muli_mle(0, my_inputs);
        let result = my_circuit.addi_muli_function(0);
        // assert_eq!(result, Fq::from(2));
        dbg!(result);
    }
}
