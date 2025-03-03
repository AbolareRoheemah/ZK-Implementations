use ark_ff::PrimeField;
use crate::bit_format::interpolate_then_evaluate_at_once;

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
                new_polys.push(interpolate_then_evaluate_at_once(poly.clone(), var_index, var_eval_at));
            }
        }
        Self::new(new_polys)
    }
    pub fn reduce(&self) -> Vec<F> {
        // Check if we have any polys
        if self.polys.is_empty() {
            return vec![F::one()]; // Return a single one if no polynomials
        }
        
        // Get the length of the first polynomial if it exists
        let arr_len = if !self.polys[0].is_empty() {
            self.polys[0].len()
        } else {
            // Default to 1 if the first polynomial is empty
            1
        };
        
        let mut reduced_polys = vec![F::one(); arr_len];
        
        for poly in &self.polys {
            // Skip empty polynomials
            if poly.is_empty() {
                continue;
            }
            
            for (i, value) in poly.iter().enumerate() {
                if i < reduced_polys.len() {
                    reduced_polys[i] *= value;
                }
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
        // Check if we have any product_polys
        if self.product_polys.is_empty() {
            return vec![F::zero()]; // Return a single zero if no polynomials
        }
        
        // Try to get the length from the first product poly if possible
        let arr_len = if !self.product_polys[0].polys.is_empty() && !self.product_polys[0].polys[0].is_empty() {
            self.product_polys[0].polys[0].len()
        } else {
            // Default to 1 if we don't have any valid polynomials
            1
        };
        
        let mut reduced_polys = vec![F::zero(); arr_len];
        
        for product_poly in &self.product_polys {
            let reduced_prod_poly = product_poly.reduce();
            
            // Make sure we handle the case where reduced_prod_poly might be shorter
            for i in 0..reduced_prod_poly.len().min(arr_len) {
                reduced_polys[i] += reduced_prod_poly[i];
            }
        }
        
        reduced_polys
    }
}