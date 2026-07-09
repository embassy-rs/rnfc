use rnfc_libnfc::Context;

fn main() {
    let ctx = Context::new();
    let mut dev = ctx.open(None).unwrap();
    println!("name: {}", dev.name());

    let _dep = dev.as_iso_dep().unwrap();
}
