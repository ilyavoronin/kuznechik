use std::ops::BitXor;

#[allow(unused)]
pub struct Galois {
    pows: Vec<u8>,
    rev_pows: Vec<u8>,
    mult_table: [[u8; 256]; 256],
    size: u8,
}

impl Galois {
    pub fn new(size: usize, gen_elem: Vec<u8>) -> Galois {
        let size = size - 1;
        let mut pows = vec![0; size];
        let mut rev_pows = vec![0_u8; 256];
        let mut curr = vec![1];
        let prim = vec![0, 1];
        for i in 0..size {
            let num = Self::get_num(&curr);
            pows[i] = num;
            rev_pows[num as usize] = i as u8;
            curr = Self::poly_mult(&curr, &prim, &gen_elem);
        }

        let mut mult_table = [[0_u8; 256]; 256];
        for i in 0..=size {
            for j in 0..=size {
                mult_table[i][j] =
                    Self::mult_internal(i as u8, j as u8, &pows, &rev_pows, size as u8);
            }
        }

        Galois {
            mult_table,
            pows,
            rev_pows,
            size: size as u8,
        }
    }

    pub fn mult(&self, a: u8, b: u8) -> u8 {
        self.mult_table[a as usize][b as usize]
    }

    pub fn sum(&self, a: u8, b: u8) -> u8 {
        a.bitxor(b)
    }

    fn mult_internal(a: u8, b: u8, pows: &Vec<u8>, rev_pows: &Vec<u8>, size: u8) -> u8 {
        if a == 0 || b == 0 {
            return 0;
        }
        //
        let c: u8 =
            ((rev_pows[a as usize] as u16 + rev_pows[b as usize] as u16) % size as u16) as u8;
        pows[c as usize]
    }

    pub fn get_num(poly: &Vec<u8>) -> u8 {
        let mut res = 0;
        for b in poly.iter().rev() {
            res = res * 2 + *b;
        }
        return res;
    }

    fn poly_mult(poly1: &Vec<u8>, poly2: &Vec<u8>, gen: &Vec<u8>) -> Vec<u8> {
        let mut res = vec![0_u8; poly1.len() + poly2.len()];

        for (i, a) in poly1.iter().enumerate() {
            for (j, b) in poly2.iter().enumerate() {
                res[i + j] = res[i + j].bitxor(a * b);
            }
        }

        Self::shrink_to_fit(&mut res);

        Self::poly_mod(&res, gen)
    }

    fn poly_mod(poly: &Vec<u8>, m: &Vec<u8>) -> Vec<u8> {
        let mdeg = Self::deg(m);
        let mut res = vec![0; poly.len()];
        let mut poly = poly.clone();
        loop {
            let curr_deg = Self::deg(&poly);
            if curr_deg < mdeg {
                break;
            }
            let diff_deg = curr_deg - mdeg;
            res[diff_deg] = 1;
            poly = Self::poly_xor(&Self::mult_x(m, diff_deg), &poly)
        }

        Self::shrink_to_fit(&mut poly);
        poly
    }

    fn mult_x(poly: &Vec<u8>, pow: usize) -> Vec<u8> {
        if pow == 0 {
            return poly.clone();
        }
        let mut res = vec![0; poly.len() + pow];

        for (i, b) in poly.iter().enumerate() {
            if *b == 1 {
                res[i + pow] = 1;
            }
        }

        Self::shrink_to_fit(&mut res);
        res
    }

    fn poly_xor(poly1: &Vec<u8>, poly2: &Vec<u8>) -> Vec<u8> {
        let size = poly2.len().max(poly1.len());
        let mut res = vec![0_u8; size];
        for i in 0..size {
            let b1 = *poly1.get(i).unwrap_or(&0);
            let b2 = *poly2.get(i).unwrap_or(&0);

            res[i] = b1.bitxor(b2);
        }

        Self::shrink_to_fit(&mut res);
        res
    }

    fn shrink_to_fit(poly: &mut Vec<u8>) {
        while poly.len() > 1 && *poly.last().unwrap() == 0_u8 {
            poly.pop();
        }
    }

    fn deg(poly: &Vec<u8>) -> usize {
        for (i, b) in poly.iter().enumerate().rev() {
            if *b != 0 {
                return i;
            }
        }
        panic!()
    }
}
