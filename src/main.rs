use num_bigint::ToBigUint;

fn main() {
    println!("Hello, world :)");
    rustls::crypto::aws_lc_rs::default_provider().install_default().expect("Failed to install rustls crypto provider"); // Init for tls
    let yeast = 1.to_biguint().unwrap();
    let _eggs = yeast.clone();
}
