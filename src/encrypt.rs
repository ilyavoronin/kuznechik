use crate::galois::Galois;
use crate::objects::{Key, BLOCK_SIZE};
use crate::poly::Polynom;

const IKEY_SIZE: usize = BLOCK_SIZE;
type IKey<'a> = &'a [u8];

static LINEAR_COEFS: &'static [u8; BLOCK_SIZE] = &[
    148, 32, 133, 16, 194, 192, 1, 251, 1, 192, 194, 16, 133, 32, 148, 1
];
static LINEAR_COEFS_DEC: &'static [u8; BLOCK_SIZE] = &[
    1, 148, 32, 133, 16, 194, 192, 1, 251, 1, 192, 194, 16, 133, 32, 148
];

pub struct Kuznechik {
    galois: Galois,
    coefs: Vec<Polynom>,
    dec_coefs: Vec<Polynom>,
}

impl Kuznechik {
    pub fn new() -> Kuznechik {
        let galois = Galois::new(256, vec![1, 1, 0, 0, 0, 0, 1, 1, 1]);
        let coefs = Self::precalc_coeffs(&galois);
        let dec_coefs = Self::precalc_coeffs_decr(&galois);

        Kuznechik {
            galois,
            coefs,
            dec_coefs,
        }
    }

    pub fn encrypt(&self, data: &mut Vec<u8>, key: Key) {
        let keys = self.get_keys(key);

        for j in (0..data.len()).step_by(BLOCK_SIZE) {
            self.xor_transform(&mut data[j..j + BLOCK_SIZE], keys[0].as_slice());
        }
        for i in 0..9 {
            let table = self.calc_table(keys[i + 1].as_slice());
            for j in (0..data.len()).step_by(BLOCK_SIZE) {
                self.apply_full_level_shifted(&mut data[j..j + BLOCK_SIZE], &table);
            }
        }
    }

    pub fn decrypt(&self, data: &mut Vec<u8>, key: Key) {
        let mut keys = self.get_keys(key);
        keys.reverse();

        for j in (0..data.len()).step_by(BLOCK_SIZE) {
            self.nonlinear_transform(&mut data[j..j + BLOCK_SIZE]);
        }

        for i in 0..9 {
            let table = self.calc_table_dec(keys[i].as_slice());
            for j in (0..data.len()).step_by(BLOCK_SIZE) {
                self.apply_dec_full_level_shifted(
                    &mut data[j..j + BLOCK_SIZE],
                    &table,
                );
            }
        }
        for j in (0..data.len()).step_by(BLOCK_SIZE) {
            self.nonlinear_transform_dec(&mut data[j..j + BLOCK_SIZE]);
            self.xor_transform(&mut data[j..j + BLOCK_SIZE], keys[9].as_slice());
        }
    }

    fn precalc_coeffs(galois: &Galois) -> Vec<Polynom> {
        let mut res = vec![Polynom::new(BLOCK_SIZE); BLOCK_SIZE];
        for i in 0..BLOCK_SIZE {
            res[i].coef[i] = 1;
        }
        for _ in 0..BLOCK_SIZE {
            let mut acc = Polynom::new(BLOCK_SIZE);
            for i in (0..BLOCK_SIZE).rev() {
                acc = acc.sum(&res[i].mult(LINEAR_COEFS[i], galois));
                if i == 0 {
                    res[i] = acc.clone();
                } else {
                    res[i] = res[i - 1].clone();
                }
            }
        }
        res
    }

    fn precalc_coeffs_decr(galois: &Galois) -> Vec<Polynom> {
        let mut res = vec![Polynom::new(BLOCK_SIZE); BLOCK_SIZE];
        for i in 0..BLOCK_SIZE {
            res[i].coef[i] = 1;
        }
        for _ in 0..BLOCK_SIZE {
            let mut acc = Polynom::new(BLOCK_SIZE);
            for i in 0..BLOCK_SIZE {
                acc = acc.sum(&res[i].mult(LINEAR_COEFS_DEC[i], galois));
                if i == BLOCK_SIZE - 1 {
                    res[i] = acc.clone();
                } else {
                    res[i] = res[i + 1].clone();
                }
            }
        }
        res
    }

    fn calc_table(&self, key: IKey) -> [u128; 4096] {
        let mut keyb = [0_u8; IKEY_SIZE];
        keyb.copy_from_slice(key);
        let keyn = u128::from_le_bytes(keyb);
        let mut res = [0_u128; 4096];
        for i in 0..BLOCK_SIZE {
            for j in 0..=255 {
                let mut mask = 0_u128;

                for k in 0..BLOCK_SIZE {
                    mask |= (self.galois.mult(NL_PERM[j], self.coefs[k].coef[i]) as u128) << (k * 8);
                }
                res[i * 256 + j as usize] = mask ^ (keyn & (255 << i * 8));
            }
        }
        res
    }

    fn calc_table_dec(&self, key: IKey) -> [u128; 4096] {
        let mut keyb = [0_u8; IKEY_SIZE];
        keyb.copy_from_slice(key);
        let keyn = u128::from_le_bytes(keyb);
        let mut res = [0_u128; 4096];
        for i in 0..BLOCK_SIZE {
            for j in 0..=255 {
                let mut mask = 0_u128;

                for k in 0..BLOCK_SIZE {
                    mask |= (self.galois.mult(
                        NL_PERM_DEC[j] ^ ((keyn >> (i * 8)) & 255) as u8,
                        self.dec_coefs[k].coef[i],
                    ) as u128)
                        << (k * 8);
                }
                res[i * 256 + j as usize] = mask;
            }
        }
        res
    }

    fn linear_xor_transform(&self, data: &mut [u8], table: &[u128; 4096]) {
        let mut acc = 0_u128;
        for i in 0..BLOCK_SIZE {
            acc ^= table[i * 256 + (data[i] as usize)]
        }

        for i in 0..BLOCK_SIZE {
            data[i] = ((acc >> (i * 8)) & 255) as u8
        }
    }

    fn linear_transform(&self, data: &mut [u8]) {
        assert_eq!(data.len(), BLOCK_SIZE);
        for _ in 0..BLOCK_SIZE {
            self.single_step_linear(data);
        }
    }

    fn single_step_linear(&self, data: &mut [u8]) {
        let mut acc = 0_u8;
        for j in (0..BLOCK_SIZE).rev() {
            acc = self
                .galois
                .sum(acc, self.galois.mult(LINEAR_COEFS[j], data[j]));
            if j == 0 {
                data[j] = acc;
            } else {
                data[j] = data[j - 1];
            }
        }
    }

    fn nonlinear_transform(&self, data: &mut [u8]) {
        assert_eq!(data.len(), BLOCK_SIZE);
        for i in 0..BLOCK_SIZE {
            data[i] = NL_PERM[data[i] as usize];
        }
    }

    fn nonlinear_transform_dec(&self, data: &mut [u8]) {
        assert_eq!(data.len(), BLOCK_SIZE);
        for i in 0..BLOCK_SIZE {
            data[i] = NL_PERM_DEC[data[i] as usize];
        }
    }

    fn xor_transform(&self, data: &mut [u8], k: IKey) {
        assert_eq!(data.len(), BLOCK_SIZE);
        for i in 0..IKEY_SIZE {
            data[i] ^= k[i];
        }
    }

    fn apply_full_level_shifted(&self, data: &mut [u8], table: &[u128; 4096]) {
        assert_eq!(data.len(), BLOCK_SIZE);
        self.linear_xor_transform(data, table);
    }

    fn apply_dec_full_level_shifted(&self, data: &mut [u8], table: &[u128; 4096]) {
        assert_eq!(data.len(), BLOCK_SIZE);
        self.linear_xor_transform(data, table);
    }

    fn apply_full_level(&self, data: &mut [u8], key: IKey) {
        assert_eq!(data.len(), BLOCK_SIZE);
        self.xor_transform(data, key);
        self.nonlinear_transform(data);
        self.linear_transform(data);
    }

    fn get_keys(&self, key: Key) -> Vec<Vec<u8>> {
        let mut ikeys = vec![vec![0_u8; IKEY_SIZE]; 10];
        ikeys[0].clone_from_slice(&key[0..IKEY_SIZE]);
        ikeys[1].clone_from_slice(&key[IKEY_SIZE..]);

        for i in 1..5 {
            let mut l = ikeys[i * 2 - 2].clone();
            let mut r = ikeys[i * 2 - 1].clone();
            for j in 0..8 {
                let mut key = vec![0_u8; IKEY_SIZE];
                key[IKEY_SIZE - 1] = ((i - 1) * 8 + j + 1) as u8;

                self.linear_transform(key.as_mut_slice());

                self.feistel(l.as_mut_slice(), r.as_mut_slice(), key.as_slice());
            }
            ikeys[i * 2] = l;
            ikeys[i * 2 + 1] = r;
        }

        ikeys
    }

    fn feistel(&self, l: &mut [u8], r: &mut [u8], k: IKey) {
        let mut tmp = vec![0_u8; IKEY_SIZE];
        tmp.clone_from_slice(r);
        r.clone_from_slice(l);
        self.apply_full_level(l, k);
        self.xor_transform(l, tmp.as_slice());
    }
}

static NL_PERM: &'static [u8; 256] = &[
    252, 238, 221, 17, 207, 110, 49, 22, 251, 196, 250, 218, 35, 197, 4, 77, 233, 119, 240, 219,
    147, 46, 153, 186, 23, 54, 241, 187, 20, 205, 95, 193, 249, 24, 101, 90, 226, 92, 239, 33, 129,
    28, 60, 66, 139, 1, 142, 79, 5, 132, 2, 174, 227, 106, 143, 160, 6, 11, 237, 152, 127, 212,
    211, 31, 235, 52, 44, 81, 234, 200, 72, 171, 242, 42, 104, 162, 253, 58, 206, 204, 181, 112,
    14, 86, 8, 12, 118, 18, 191, 114, 19, 71, 156, 183, 93, 135, 21, 161, 150, 41, 16, 123, 154,
    199, 243, 145, 120, 111, 157, 158, 178, 177, 50, 117, 25, 61, 255, 53, 138, 126, 109, 84, 198,
    128, 195, 189, 13, 87, 223, 245, 36, 169, 62, 168, 67, 201, 215, 121, 214, 246, 124, 34, 185,
    3, 224, 15, 236, 222, 122, 148, 176, 188, 220, 232, 40, 80, 78, 51, 10, 74, 167, 151, 96, 115,
    30, 0, 98, 68, 26, 184, 56, 130, 100, 159, 38, 65, 173, 69, 70, 146, 39, 94, 85, 47, 140, 163,
    165, 125, 105, 213, 149, 59, 7, 88, 179, 64, 134, 172, 29, 247, 48, 55, 107, 228, 136, 217,
    231, 137, 225, 27, 131, 73, 76, 63, 248, 254, 141, 83, 170, 144, 202, 216, 133, 97, 32, 113,
    103, 164, 45, 43, 9, 91, 203, 155, 37, 208, 190, 229, 108, 82, 89, 166, 116, 210, 230, 244,
    180, 192, 209, 102, 175, 194, 57, 75, 99, 182,
];

static NL_PERM_DEC: &'static [u8; 256] = &[
    165, 45, 50, 143, 14, 48, 56, 192, 84, 230, 158, 57, 85, 126, 82, 145, 100, 3, 87, 90, 28, 96,
    7, 24, 33, 114, 168, 209, 41, 198, 164, 63, 224, 39, 141, 12, 130, 234, 174, 180, 154, 99, 73,
    229, 66, 228, 21, 183, 200, 6, 112, 157, 65, 117, 25, 201, 170, 252, 77, 191, 42, 115, 132,
    213, 195, 175, 43, 134, 167, 177, 178, 91, 70, 211, 159, 253, 212, 15, 156, 47, 155, 67, 239,
    217, 121, 182, 83, 127, 193, 240, 35, 231, 37, 94, 181, 30, 162, 223, 166, 254, 172, 34, 249,
    226, 74, 188, 53, 202, 238, 120, 5, 107, 81, 225, 89, 163, 242, 113, 86, 17, 106, 137, 148,
    101, 140, 187, 119, 60, 123, 40, 171, 210, 49, 222, 196, 95, 204, 207, 118, 44, 184, 216, 46,
    54, 219, 105, 179, 20, 149, 190, 98, 161, 59, 22, 102, 233, 92, 108, 109, 173, 55, 97, 75, 185,
    227, 186, 241, 160, 133, 131, 218, 71, 197, 176, 51, 250, 150, 111, 110, 194, 246, 80, 255, 93,
    169, 142, 23, 27, 151, 125, 236, 88, 247, 31, 251, 124, 9, 13, 122, 103, 69, 135, 220, 232, 79,
    29, 78, 4, 235, 248, 243, 62, 61, 189, 138, 136, 221, 205, 11, 19, 152, 2, 147, 128, 144, 208,
    36, 52, 203, 237, 244, 206, 153, 16, 68, 64, 146, 58, 1, 38, 18, 26, 72, 104, 245, 129, 139,
    199, 214, 32, 10, 8, 0, 76, 215, 116,
];

#[cfg(test)]
mod tests {
    use crate::encrypt::Kuznechik;
    use crate::objects::{Key, KEY_SIZE};

    #[test]
    fn test_gost_34_12_2018() {
        let k = Kuznechik::new();

        let mut key: Key = [0_u8; KEY_SIZE];
        key.copy_from_slice(hex::decode("8899aabbccddeeff0011223344556677fedcba98765432100123456789abcdef").unwrap().as_slice());
        let mut data = hex::decode("1122334455667700ffeeddccbbaa9988").unwrap();

        k.encrypt(&mut data, key);
        assert_eq!("7f679d90bebc24305a468d42b9d4edcd", hex::encode(&data));

        k.decrypt(&mut data, key);

        assert_eq!("1122334455667700ffeeddccbbaa9988", hex::encode(data));
    }
}