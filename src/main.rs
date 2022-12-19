extern crate core;

use crate::encrypt::Kuznechik;
use crate::objects::{gen_data, gen_key};
use crate::time::Timer;
use rand::prelude::StdRng;
use rand::SeedableRng;

mod encrypt;
mod galois;
mod objects;
mod poly;
mod time;

fn main() {
    let data_size_mb = 100;

    let mut timer = Timer::new();
    let kuznechik = Kuznechik::new();

    let mut rng = StdRng::seed_from_u64(42);
    let gen_data = gen_data(data_size_mb * 1024 * 1024, &mut rng);
    let key = gen_key(&mut rng);
    let mut data = gen_data.clone();

    timer.start("encrypt");
    kuznechik.encrypt(&mut data, key);
    timer.finish("encrypt");

    timer.start("decrypt");
    kuznechik.decrypt(&mut data, key);
    timer.finish("decrypt");

    assert_eq!(gen_data, data);

    let encoding_time_ms = timer.get_res("encrypt");
    let decoding_time_ms = timer.get_res("decrypt");
    println!(
        "Total time ms: {}, Encryption speed: {} Mb/s",
        encoding_time_ms,
        data_size_mb as f64 * 1_000.0 / encoding_time_ms as f64
    );
    println!(
        "Total time ms: {}, Decryption speed: {} Mb/s",
        decoding_time_ms,
        data_size_mb as f64 * 1_000.0 / decoding_time_ms as f64
    );
}
