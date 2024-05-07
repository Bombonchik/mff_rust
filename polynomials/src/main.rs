use std::collections::HashMap;
use std::ops::Add;
use std::cmp::PartialEq;

pub struct Polynomial {
    polinomial: HashMap<String, HashMap<i32, i64>>,
}

impl Polynomial {
    pub fn builder() -> PolynomialBuilder {
        PolynomialBuilder::default()
    }

    fn add_monomial(&mut self, coefficient: i64, term: &str, exponent: i32)  {
        let terms = self.polinomial.get_mut(term).unwrap();
        let target_coefficient = terms.get_mut(&exponent);
        match target_coefficient {
            Some(target) => {
                *target += coefficient;
            },
            None => {
                terms.insert(exponent, coefficient);
            }
        }
    }

    fn clear_zero_terms(&mut self) {
        let mut terms_to_remove = Vec::new();
        for (term, value) in &mut self.polinomial {
            let mut exponents_to_remove = Vec::new();
            for (exponent, coefficient) in value.iter() {
                if *coefficient == 0 {
                    exponents_to_remove.push(*exponent);
                }
            }
            
            for exponent in exponents_to_remove {
                value.remove(&exponent);
            }

            if value.is_empty() {
                terms_to_remove.push(term.clone());
            }
        }
        for term in terms_to_remove {
            self.polinomial.remove(&term);
        }
    }
}

impl Add for Polynomial {
    type Output = Polynomial;

    fn add(self, other: Polynomial) -> Polynomial {
        let mut result = Polynomial::builder().build();
        result.polinomial = self.polinomial.clone();
        for (key, value) in other.polinomial {
            let similar_terms = self.polinomial.get(&key);
            match similar_terms {
                Some(target) => {
                    for (exponent, coefficient) in value {
                        result.add_monomial(coefficient, &key, exponent);
                    }
                },
                None => {
                    result.polinomial.insert(key, value);
                }
            };
            
        }
        result.clear_zero_terms();
        result
    }
}

impl PartialEq for Polynomial {
    fn eq(&self, other: &Polynomial) -> bool {
        for (key, value) in &self.polinomial {
            let similar_terms = other.polinomial.get(key);
            match similar_terms {
                Some(target) => {
                    for (exponent, coefficient) in value {
                        let target_coefficient = target.get(exponent);
                        match target_coefficient {
                            Some(target) => {
                                if target != coefficient {
                                    return false;
                                }
                            },
                            None => {
                                return false;
                            }
                        }
                    }
                },
                None => {
                    return false;
                }
            };
            
        }
        true
    }
}
#[derive(Default)]
pub struct PolynomialBuilder {
    polinomial: HashMap<String, HashMap<i32, i64>>,
}

impl PolynomialBuilder {
    pub fn new() -> PolynomialBuilder {
        PolynomialBuilder {
            polinomial: HashMap::new(),
        }
    }

    pub fn add(mut self, coefficient: i64, term: &str, exponent: i32) -> Self {
        let similar_terms = self.polinomial.get_mut(term);
        match similar_terms {
            Some(terms) => {
                let target_coefficient = terms.get_mut(&exponent);
                match target_coefficient {
                    Some(target) => {
                        *target += coefficient;
                    },
                    None => {
                        terms.insert(exponent, coefficient);
                    }
                }
            },
            None => {
                let mut exponent_to_coefficient = HashMap::new();
                exponent_to_coefficient.insert(exponent, coefficient);
                self.polinomial.insert(term.to_string(), exponent_to_coefficient);
            }
        }
        self
    }

    pub fn build(self) -> Polynomial {
        let mut polynomial = Polynomial {
            polinomial: self.polinomial
        };
        polynomial.clear_zero_terms();
        polynomial
    }
}


fn main() {
    let a = Polynomial::builder()
        .add(1, "x", 2)
        .add(4, "x", 5)
        .add(2, "x", 2)
        .add(3, "y", 3)
        .build();
    //let builder = builder.add(1, "x", 2).add(4, "x", 5);
    //let builder = builder.add(2, "x", 2).add(3, "y", 3);
    let b = Polynomial::builder()
        .add(10, "x", 3)
        .add(-2, "x", 5)
        .add(2, "z", 2)
        .add(3, "z", 3)
        .build();
    assert!(a != b);
    let c = a + b;
    let a = Polynomial::builder()
        .add(3, "x", 3)
        .add(3, "y", 5)
        .add(4, "y", 4)
        .build();
}
