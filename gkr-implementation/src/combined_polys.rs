use ark_ff::PrimeField;
use crate::bit_format::evaluate_interpolate;

#[derive(Debug, Clone)]
pub struct ProductPoly<F: PrimeField> {
    pub polys: Vec<Vec<F>>
}

impl <F: PrimeField>ProductPoly<F> {
    pub fn new(polys: Vec<Vec<F>>) -> Self {
        Self {
            polys
        }
    }

    pub fn degree(&self) -> usize {
        self.polys.len()
    }

    pub fn partial_evaluate(&self, var_index: usize, var_eval_at: F) -> Self {
        let mut new_polys: Vec<Vec<F>> = vec![];
        for poly in &self.polys {
            if poly.len() != 1 {
                new_polys.push(evaluate_interpolate(poly.clone(), var_index, var_eval_at));
            }
        }
        Self::new(new_polys)
    }
    pub fn reduce(&self) -> Vec<F> {
        let mut reduced_polys = vec![F::one(); self.polys[0].len()]; // Initialize with zeros

        for poly in &self.polys {
            for (i, value) in poly.iter().enumerate() {
                reduced_polys[i] *= value; 
            }
        }
        reduced_polys
    }
}

#[derive(Debug, Clone)]
pub struct SumPoly<F: PrimeField> {
    pub product_polys: Vec<ProductPoly<F>>
}

impl <F: PrimeField>SumPoly<F> {
    pub fn new(product_polys: Vec<ProductPoly<F>>) -> Self {
        Self {
            product_polys
        }
    }

    pub fn degree(&self) -> usize {
        self.product_polys.len()
    }

    pub fn partial_evaluate(&self, var_index: usize, var_eval_at: F) -> Self {
        let mut new_product_polys: Vec<ProductPoly<F>> = vec![];
        for product_poly in &self.product_polys {
            new_product_polys.push(product_poly.partial_evaluate(var_index, var_eval_at));
        }
        Self::new(new_product_polys)
    }

    pub fn reduce(&self) -> Vec<F> {
        let arr_len = self.product_polys[0].polys[0].len();
        let mut reduced_polys = vec![F::zero(); arr_len];
        // let mut reduced_polys_arr = vec![F::zero(); ];

        for product_poly in &self.product_polys {
            let reduced_prod_poly = product_poly.reduce();
            // reduced_polys_arr.push(reduced_prod_poly);
            for i in 0..reduced_prod_poly.len() {
                reduced_polys[i] += reduced_prod_poly[i];
            }
        }
        reduced_polys
    }
}