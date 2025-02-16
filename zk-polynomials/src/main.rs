fn main() {
    struct Univariatepoly {
        coef: Vec<u32>
    }

    impl Univariatepoly{
        fn new(coef: Vec<u32>) -> Univariatepoly {
            Univariatepoly {
                coef
            }
        }

        fn degree(&self) -> usize {
            self.coef.len() - 1
        }
                                    
        fn evaluate(&self, x: u32) -> u32 {
            let mut target_result: u32 = 0;
            let mut current_x = 1;
            for i in 0..self.coef.len() {
                // println!("{}", i);
                target_result += self.coef[i] * x.pow(i.try_into().unwrap())
            }
            target_result
        }

        fn another_evaluate(&self, x: u32) -> u32 {
            self.coef.iter().enumerate().map(|(i, coeff)| {
                coeff * x.pow(i as u32)
            }).sum()
            // self.coef.iter().rev().reduce(|acc, curr| acc * x + curr).unwrap()
        }

        fn interpolate(xs: Vec<u32>, ys: Vec<u32>) -> Self {
            let point_iter = xs.iter().zip(ys.iter());
            point_iter.map(|(x, y)| {
                basis(x, &xs).scalar_mul(y)
            }).sum()
        }

        fn scalar_mul(&self, scalar: &u32) -> Self {
            Univariatepoly {
                coef: self.coef.iter().map(|coeff| coeff * scalar).collect()
            }
        }

        // fn interpolate(&self, value_vec: Vec<u32>) -> Vec<u32> {
        //     let my_value_arr: Vec<u32>;
        //     for i in 0..value_vec.len() {
        //         let my_evaluation = evaluate(self, value_vec[i]);
        //         my_value_arr.push(my_evaluation);
        //     }
        //     let my_lagrange_arr: Vec<u32>;
        //     for i in 0..my_value_arr.len() {

        //     }
        // }
    }
}
