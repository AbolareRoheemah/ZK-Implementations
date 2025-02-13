fn main() {

}

// I need to import the transcript from the fiat shamir implementation
// I need to modify the prover and verify. Instead if them working with a single polynomial, they would now work with an array of polynomials
// Prover:
// It now takes in a vector that takes in two productPoly structs 
// and each productpoly contains two array of evaluations. We need to be able to do sum check on each 
// vec in the array of evals. You know the prev sum check protocol only took in one array of evals

// Verifier:
