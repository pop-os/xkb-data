fn main() {
    println!("Keyboard layouts");
    let layouts = xkb_data::keyboard_layouts().unwrap();
    let mut count = 0;
    for layout in layouts.layouts() {
        println!("  {}: {}", layout.name(), layout.description());
        if let Some(variants) = layout.variants() {
            for variant in variants {
                println!("    {}: {}", variant.name(), variant.description())
            }
        }
        count += 1;
    }

    println!("Total layouts without extra sources: {}", count);

    count = 0;
    let all_layouts = xkb_data::all_keyboard_layouts().unwrap();
    for layout in all_layouts.layouts() {
        println!("  {}: {}", layout.name(), layout.description());
        if let Some(variants) = layout.variants() {
            for variant in variants {
                println!("    {}: {}", variant.name(), variant.description())
            }
        }
        count += 1;
    }
    println!("Total layouts including extra sources: {}", count);
}
