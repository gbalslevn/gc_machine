use gc_machine::ot::ot;

fn main() {
    println!("Hello, world :)");
    let pp = ot::PublicParameters::new();
    let p = pp.get_p();
    println!("{}", p);
}
