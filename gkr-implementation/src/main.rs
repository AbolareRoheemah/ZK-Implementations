use ark_ff::{BigInteger, PrimeField};
pub mod bit_format;
pub mod univar_poly;
pub mod gkr_sum_check;
pub mod fiat_shamir_transcript;
pub mod combined_polys;
use combined_polys::SumPoly;
use bit_format::{interpolate_then_evaluate_at_once, full_evaluate};
use sha3::Digest;
use fiat_shamir_transcript::{conv_poly_to_byte, Transcript};
pub mod gkr_circuit;
use gkr_circuit::{Circuit, Layer};
fn main() {
    // println!("Hello, world!");
}

#[derive(Debug)]
struct GKRProof<F: PrimeField>  {
    w_poly: Vec<F>,
    sumcheck_proofs: Vec<gkr_sum_check::Proof<F>>,
    w_rb_rcs: Vec<(F, F)>
}

impl <F:PrimeField>GKRProof<F> {
    fn init(w_poly: Vec<F>, sumcheck_proofs: Vec<gkr_sum_check::Proof<F>>, w_rb_rcs: Vec<(F, F)>) -> Self {
        Self {
            w_poly, //W0
            sumcheck_proofs,
            w_rb_rcs // [(wb, wc), (wb, wc)]
        }
    }
}

struct GKRProver<F: PrimeField>  {
    transcript: Transcript<sha3::Keccak256, F>,
    circuit: Circuit<F>
}

impl <F: PrimeField>GKRProver<F> {
    fn init(transcript: Transcript<sha3::Keccak256, F>, circuit: Circuit<F>) -> Self {
        Self {
            transcript,
            circuit
        }
    }

    fn execute_circuit(&mut self, inputs: Vec<F>) -> &Vec<Layer<F>> {
        self.circuit.execute(inputs)
    }

    fn evaluate_fbc_poly(&mut self, initial_fbc_poly: SumPoly<F>, eval_points: Vec<F>) -> F {
        let mut fbc_poly = initial_fbc_poly;
        // let mut fbc_poly_eval = vec![];
        for i in 0..eval_points.len() {
            fbc_poly = fbc_poly.partial_evaluate(0, eval_points[i]);
        }
        fbc_poly.reduce()[0]
    }

    pub fn multilinearize_w_poly(&mut self) -> Vec<&mut Vec<F>> {
        let circuit_layers = &mut self.circuit.layers; // Change to mutable reference
        let mut output = vec![];

        for layer in circuit_layers.iter_mut() {
            let poly = &mut layer.layer_output;

            // Check if poly length is a power of 2
            if poly.is_empty() || (poly.len() & (poly.len() - 1)) != 0 {
                // Calculate the next power of 2
                let next_power_of_2 = 1 << (poly.len() as f64).log2().ceil() as usize;

                // Pad with zeros to reach the next power of 2
                poly.resize(next_power_of_2, F::zero());
            }

            output.push(poly); // Ensure output is populated
        }
        output
    }

    fn prove(&mut self) -> GKRProof<F> {
        let gkr_transcript = &mut self.transcript; // pass

        let circuit = &mut self.circuit.clone(); //pass

        // step1
        let w_0 = &circuit.layers[0].layer_output; //pass
        let binding = conv_poly_to_byte(w_0); //pass
        gkr_transcript.absorb(binding.as_slice()); // pass

        let (output_bit, _) = circuit.get_bit_len(0); // pass
        // step2
        let r_0 = gkr_transcript.squeeze_n(output_bit as usize); //pass

        let mut initial_claimed_sum_m0 = full_evaluate(&r_0.len(), w_0.to_vec(), &r_0); //pass
        // let proof_w0 = w_0.clone();

        // step3
        let mut initial_fbc_polynomial = self.circuit.get_fbc_poly(0, r_0); //pass


        let mut sumcheck_proofs = vec![];
        let mut w_rb_rcs = vec![];
        for i in 0..circuit.layers.len() {
            println!("Layer {}", i);
            let (_, total_bc_bits) = circuit.get_bit_len(i); 
            // step4 - Initializing prover
            let mut sumcheck_transcript = Transcript::new(sha3::Keccak256::new()); //pass
            let mut sumcheck_prover = gkr_sum_check::Prover::init(initial_fbc_polynomial.clone(), sumcheck_transcript.clone(), initial_claimed_sum_m0); // pass

            let sumcheck_proof = sumcheck_prover.generate_gkr_sumcheck_proof(
                initial_fbc_polynomial.clone(),
                initial_claimed_sum_m0,
                total_bc_bits
            );
            let r_values = &sumcheck_proof.rand_challenges;
            sumcheck_proofs.push(sumcheck_proof.clone());

            // step5 - getting new fbc and claimed sum
            let (addi_poly, muli_poly) = self.circuit.addi_muli_function(i + 1);
            // Wi + 1 for claimed sum
            let wb = self.circuit.get_wi_x_func(i + 1, 1);
            let wc = self.circuit.get_wi_x_func(i + 1, 0);
            let (new_fbc_poly, new_claimed_sum, w_rb, w_rc) = self.circuit.get_alpha_beta_poly_and_sum(i + 1, addi_poly.to_vec(), muli_poly.to_vec(), wb, wc, r_values.to_vec(), &mut sumcheck_transcript);
            w_rb_rcs.push((w_rb[0], w_rc[0]));
            initial_fbc_polynomial = new_fbc_poly;
            initial_claimed_sum_m0 = new_claimed_sum;
        }
        GKRProof {
            sumcheck_proofs,
            w_poly: w_0.to_vec(),
            w_rb_rcs
        }
    }

}


struct GkrVerifier<F: PrimeField> {
    transcript: Transcript<sha3::Keccak256, F>,
    circuit: Circuit<F>
}

impl <F: PrimeField> GkrVerifier<F> {
    fn init(transcript: Transcript<sha3::Keccak256, F>, circuit: Circuit<F>) -> Self {
        Self {
            transcript,
            circuit
        }
    }

    fn evaluate_input_layer(&self, inputs: &Vec<F>, rand_chal_arr: &Vec<F>) -> (F, F) {
        let (rb, rc) = rand_chal_arr.split_at(rand_chal_arr.len() / 2);
        let wb = full_evaluate(&rb.len(), inputs.to_vec(), &rb.to_vec());
        let wc = full_evaluate(&rc.len(), inputs.to_vec(), &rc.to_vec());

        (wb, wc)
    }

    fn verify_gkr_proof(&mut self, gkr_proof: GKRProof<F>, inputs: Vec<F>) -> bool {
        let gkr_transcript = &mut self.transcript.clone();

        let sumcheck_proof = &gkr_proof.sumcheck_proofs;

        let w_0 = gkr_proof.w_poly;

        let binding = conv_poly_to_byte(&w_0);
        gkr_transcript.absorb(binding.as_slice());

        let circuit_layers = self.circuit.layers.len();
        for i in 0..circuit_layers {
            let (output_bit, _) = self.circuit.get_bit_len(i);
    
            let r_0 = gkr_transcript.squeeze_n(output_bit as usize);
            
            // initializing verifier
            let mut sumcheck_verifier = gkr_sum_check::GKRSumcheckVerifier::init(sumcheck_proof[i].clone());

            let (rand_chal_arr, claimed_sum) = sumcheck_verifier.verify_gkr_sumcheck_proof();

            let (w_rb, w_rc) = if i == circuit_layers - 1 {
                self.evaluate_input_layer(&inputs, &rand_chal_arr)
            } else {
                gkr_proof.w_rb_rcs[i]
            };
            let (mut addi_poly, mut muli_poly) = self.circuit.addi_muli_function(i);
            for r in &r_0 {
                addi_poly = interpolate_then_evaluate_at_once(addi_poly, 0, *r);
                muli_poly = interpolate_then_evaluate_at_once(muli_poly, 0, *r);
            }
            let addi_poly_evaluated = full_evaluate(&rand_chal_arr.len(), addi_poly, &rand_chal_arr);
            let muli_poly_evaluated = full_evaluate(&rand_chal_arr.len(), muli_poly, &rand_chal_arr);

            let evaluated_claimed_sum = if i == 0 {
                addi_poly_evaluated * (w_rb + w_rc) + muli_poly_evaluated * (w_rb * w_rc)
            } else {
                gkr_transcript.absorb(&w_rb.into_bigint().to_bytes_be().as_slice());
                let alpha = gkr_transcript.squeeze();
    
                gkr_transcript.absorb(&w_rc.into_bigint().to_bytes_be().as_slice());
                let beta = gkr_transcript.squeeze();

                let new_addi = alpha * addi_poly_evaluated + beta * addi_poly_evaluated;
                let new_muli = alpha * muli_poly_evaluated + beta * muli_poly_evaluated;

                new_addi * (w_rb + w_rc) + new_muli * (w_rb * w_rc)
            };
            if claimed_sum != evaluated_claimed_sum {
                return false;
            }
        }
        true
    }

}



#[cfg(test)]
mod test {
    use super::*;
    use ark_bn254::Fq;

    #[test]
    fn test_gkr_prover_prove() {
        // Setup a finite field and a circuit
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

        // Initialize the prover
        let transcript = Transcript::new(sha3::Keccak256::new());
        let mut gkr_prover = GKRProver::init(transcript, my_circuit);

        // Execute the prove function
        let proof = gkr_prover.prove();

        println!("gkr proof{:?}", proof);
    }

    #[test]
    fn test_gkr_verifier() {
        // Setup a finite field and a circuit
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
        my_circuit.execute(my_inputs.clone());

        // Initialize the verifier
        let transcript = Transcript::new(sha3::Keccak256::new());
        let mut gkr_verifier = GkrVerifier::init(transcript, my_circuit.clone());

        // Initialize the prover
        let transcript = Transcript::new(sha3::Keccak256::new());
        let mut gkr_prover = GKRProver::init(transcript, my_circuit);
        let proof = gkr_prover.prove();

        // Execute the verify function
        let verified = gkr_verifier.verify_gkr_proof(proof, my_inputs);

        println!("gkr verified {}", verified);
    }

}