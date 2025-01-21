fn main() {
    struct Univariatepoly {
        coef: Vec<(u32, u32)>
    }

    impl Univariatepoly {
        fn new(coef: Vec<(u32, u32)>) -> Univariatepoly {
            Univariatepoly {
                coef
            }
        }

        fn degree(&self) -> u32 {
            let mut my_degree: u32 = 0;
            for i in 0..self.coef.len() {
                let (_, exp) = self.coef[i];
                if exp > my_degree {
                    my_degree = exp
                }
            };
            my_degree
        }

        fn evaluate(&self, x: u32) -> u32 {
            self.coef.iter().map(|(coef, exp)| *coef * x.pow(*exp)).sum()
        }
    }

    // f(x) = 3x^2 + 5
    let my_struct = Univariatepoly {
        coef: vec![(5, 0), (3, 2)]
    };

    let my_degree = my_struct.degree();
    println!("{}", my_degree);

    let my_eval = my_struct.evaluate(1);
    println!("{}", my_eval);
}