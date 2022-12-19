use rand::RngCore;

pub const BLOCK_SIZE: usize = 16;
pub const KEY_SIZE: usize = 32;
pub type Key = [u8; KEY_SIZE];

pub fn gen_key(rng: &mut impl RngCore) -> Key {
    let mut key = [0_u8; KEY_SIZE];
    rng.fill_bytes(key.as_mut_slice());

    key
}

pub fn gen_data(size_bytes: usize, rng: &mut impl RngCore) -> Vec<u8> {
    let mut data: Vec<u8> = vec![0; size_bytes];
    rng.fill_bytes(data.as_mut_slice());

    data
}
