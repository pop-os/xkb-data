fn main() {
    println!("Keyboard layouts");
    let layouts = xkb_data::keyboard_layouts().unwrap();
    for layout in layouts.layouts() {
        println!("  {}: {}", layout.name(), layout.description());
        if let Some(variants) = layout.variants() {
            for variant in variants {
                println!("    {}: {}", variant.name(), variant.description())
            }
        }
    }
}