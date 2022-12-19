use crate::galois::Galois;
use std::ops::BitXor;

#[derive(Clone)]
pub struct Polynom {
    pub coef: Vec<u8>,
}

impl Polynom {
    pub fn new(size: usize) -> Polynom {
        Polynom {
            coef: vec![0_u8; size],
        }
    }

    pub fn from_bytes(b: &[u8]) -> Polynom {
        return Polynom { coef: Vec::from(b) };
    }

    pub fn mult(&self, m: u8, g: &Galois) -> Polynom {
        let mut res = Polynom::from_bytes(self.coef.as_slice());
        for b in res.coef.iter_mut() {
            *b = g.mult(*b, m);
        }
        res
    }

    pub fn sum(&self, other: &Polynom) -> Polynom {
        let mut res = Polynom::from_bytes(self.coef.as_slice());
        for i in 0..res.coef.len() {
            res.coef[i] = res.coef[i].bitxor(other.coef[i]);
        }
        res
    }

    #[allow(unused)]
    pub fn substitute(&self, v: &[u8], g: &Galois) -> u8 {
        let mut res = 0_u8;
        for i in 0..self.coef.len() {
            res ^= g.mult(v[i], self.coef[i]);
        }
        res
    }
}
