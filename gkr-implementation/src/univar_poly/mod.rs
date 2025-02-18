use std::iter::{Product, Sum};
use std::ops::{Add, Mul};
use ark_ff::PrimeField;

#[derive(Debug, PartialEq, Clone)]
pub struct Univariatepoly<F: PrimeField> {
    pub coef: Vec<F>
}

impl <F: PrimeField> Univariatepoly<F> {
    pub fn new(coef: Vec<F>) -> Self {
        Univariatepoly {
            coef
        }
    }

    fn degree(&self) -> usize {
        self.coef.len() - 1
    }

    pub fn evaluate(&self, x: F) -> F {
        // self.coef.iter().enumerate().map(|(index, val)| val * x.pow(index as f64)).sum()
        self.coef.iter().rev().cloned().reduce(|acc, curr| acc * x + curr).unwrap()
    }

   pub fn interpolate(xs: Vec<F>, ys: Vec<F>) -> Self {
        xs.iter().zip(ys.iter()).map(|(x, y)| Self::basis(x, xs.clone()).scalar_mul(*y)).sum()
    }

    fn scalar_mul(&self, scalar: F) -> Self {
        let new_coef = self.coef.iter().map(|coef| *coef * scalar).collect();
        Univariatepoly {
            coef: new_coef
        }
    }
    
    fn basis(x: &F, interpolating_set: Vec<F>) -> Self {
        // numerator
        let numerator: Univariatepoly<F> = interpolating_set.iter().filter(|x_val| *x_val != x).map(|x_in_set| Univariatepoly::new(vec![-*x_in_set, F::one()])).product();
    
        // denominator
        let denominator = F::one() / numerator.evaluate(*x);
        numerator.scalar_mul(denominator)
    }
}

impl <F: PrimeField> Add for &Univariatepoly<F> {
    type Output = Univariatepoly<F>;

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

impl <F: PrimeField> Sum for Univariatepoly<F> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut new_sum = Univariatepoly::new(vec![F::zero()]);
        for poly in iter {
            new_sum = &new_sum + &poly 
        };
        new_sum
    } 
}

impl <F: PrimeField> Mul for &Univariatepoly<F> {
    type Output = Univariatepoly<F>;

    fn mul(self, rhs: Self) -> Self::Output {
        let new_degree = self.degree() + rhs.degree();
        let mut result = vec![F::zero(); new_degree + 1];
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

impl <F: PrimeField> Product for Univariatepoly<F> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = Univariatepoly::new(vec![F::one()]);
        for poly in iter {
            result = &result * &poly;
        }
        result
    }
}

#[cfg(test)]
mod test {
    use ark_ff::PrimeField;
    use ark_bn254::Fq;
    use crate::univar_poly::Univariatepoly;

    fn poly_1<F: PrimeField>() -> Univariatepoly<F> {
        // f(x) = 1 + 2x + 3x^2
        Univariatepoly {
            coef: vec![F::from(1), F::from(2), F::from(3)]
        }
    }

    fn poly_2<F: PrimeField>() -> Univariatepoly<F> {
        // f(x) = 4x + 3 + 5x^11
        Univariatepoly {
            coef: [vec![F::from(3), F::from(4)], vec![F::from(0); 9], vec![F::from(5)]].concat(),
        }
    }

    #[test]
    fn test_degree() {
        assert_eq!(poly_1::<Fq>().degree(), 2);
    }

    #[test]
    fn test_evaluate() {
        assert_eq!(poly_1().evaluate(Fq::from(2)), Fq::from(17))
    }

    #[test]
    fn test_addition() {
        assert_eq!((&poly_1::<F>() + &poly_2()).coef, [vec![Fq::from(4), Fq::from(6), Fq::from(3)], vec![Fq::from(0); 8], vec![Fq::from(5)]].concat())
    }

    #[test]
    fn test_mul() {
        // f(x) = 5 + 2x^2
        let poly_1 = Univariatepoly {
            coef: vec![Fq::from(5), Fq::from(0), Fq::from(2)],
        };
        // f(x) = 2x + 6
        let poly_2 = Univariatepoly {
            coef: vec![Fq::from(6), Fq::from(2)],
        };

        assert_eq!((&poly_1 * &poly_2).coef, vec![Fq::from(30), Fq::from(10), Fq::from(12), Fq::from(4)]);
    }

    #[test]
    fn test_interpolate() {
        let ans = Univariatepoly::interpolate(vec![Fq::from(2), Fq::from(4)], vec![Fq::from(4), Fq::from(8)]);

        assert_eq!(ans.coef, vec![Fq::from(0), Fq::from(2)]);
    }
}

fn main() {

}