use std::iter::{Product, Sum};
use std::ops::{Add, Mul};

#[derive(Debug, PartialEq, Clone)]
pub struct Univariatepoly {
    coef: Vec<f64>
}

impl Univariatepoly {
    pub fn new(coef: Vec<f64>) -> Self {
        Univariatepoly {
            coef
        }
    }

    fn degree(&self) -> usize {
        self.coef.len() - 1
    }

    fn evalaute(&self, x: f64) -> f64 {
        // self.coef.iter().enumerate().map(|(index, val)| val * x.pow(index as f64)).sum()
        self.coef.iter().rev().cloned().reduce(|acc, curr| acc * x + curr).unwrap()
    }

   pub fn interpolate(xs: Vec<f64>, ys: Vec<f64>) -> Self {
        xs.iter().zip(ys.iter()).map(|(x, y)| Self::basis(x, xs.clone()).scalar_mul(*y)).sum()
    }

    fn scalar_mul(&self, scalar: f64) -> Self {
        let new_coef = self.coef.iter().map(|coef| *coef * scalar).collect();
        Univariatepoly {
            coef: new_coef
        }
    }
    
    fn basis(x: &f64, interpolating_set: Vec<f64>) -> Self {
        // numerator
        let numerator: Univariatepoly = interpolating_set.iter().filter(|x_val| *x_val != x).map(|x_in_set| Univariatepoly::new(vec![-x_in_set, 1.0])).product();
    
        // denominator
        let denominator = 1.0 / numerator.evalaute(*x);
        numerator.scalar_mul(denominator)
    }
}

impl Add for &Univariatepoly {
    type Output = Univariatepoly;

    fn add(self, rhs: Self) -> Self::Output {
        let (mut bigger, smaller) = if self.degree() < rhs.degree() {
            (rhs.clone(), self)
        } else {
            (self.clone(), rhs)
        };

        let _ = bigger.coef.iter_mut().zip(smaller.coef.iter()).map(|(big, sm)| *big += sm).collect::<()>();

        Univariatepoly::new(bigger.coef)
    }
}

impl Sum for Univariatepoly {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut new_sum = Univariatepoly::new(vec![0.0]);
        for poly in iter {
            new_sum = &new_sum + &poly 
        };
        new_sum
    } 
}

impl Mul for &Univariatepoly {
    type Output = Univariatepoly;

    fn mul(self, rhs: Self) -> Self::Output {
        let new_degree = self.degree() + rhs.degree();
        let mut result = vec![0.0; new_degree + 1];
        for i in 0..self.coef.len() {
            for j in 0..rhs.coef.len() {
                result[i + j] += self.coef[i] * rhs.coef[j]
            }
        };
        Univariatepoly {
            coef: result
        }
    }
}

impl Product for Univariatepoly {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = Univariatepoly::new(vec![1.0]);
        for poly in iter {
            result = &result * &poly;
        }
        result
    }
}

#[cfg(test)]
mod test {
    use crate::Univariatepoly;

    fn poly_1() -> Univariatepoly {
        // f(x) = 1 + 2x + 3x^2
        Univariatepoly {
            coef: vec![1.0, 2.0, 3.0]
        }
    }

    fn poly_2() -> Univariatepoly {
        // f(x) = 4x + 3 + 5x^11
        Univariatepoly {
            coef: [vec![3.0, 4.0], vec![0.0; 9], vec![5.0]].concat(),
        }
    }

    #[test]
    fn test_degree() {
        assert_eq!(poly_1().degree(), 2);
    }

    #[test]
    fn test_evaluate() {
        assert_eq!(poly_1().evalaute(2.0), 17.0)
    }

    #[test]
    fn test_addition() {
        assert_eq!((&poly_1() + &poly_2()).coef, [vec![4.0, 6.0, 3.0], vec![0.0; 8], vec![5.0]].concat())
    }

    #[test]
    fn test_mul() {
        // f(x) = 5 + 2x^2
        let poly_1 = Univariatepoly {
            coef: vec![5.0, 0.0, 2.0],
        };
        // f(x) = 2x + 6
        let poly_2 = Univariatepoly {
            coef: vec![6.0, 2.0],
        };

        assert_eq!((&poly_1 * &poly_2).coef, vec![30.0, 10.0, 12.0, 4.0]);
    }

    #[test]
    fn test_interpolate() {
        let ans = Univariatepoly::interpolate(vec![2.0, 4.0], vec![4.0, 8.0]);

        assert_eq!(ans.coef, vec![0.0, 2.0]);
    }
}

fn main() {

}