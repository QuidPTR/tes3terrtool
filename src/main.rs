use clap::Parser;
use tes3::esp::{Plugin, Landscape};


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    ///What to do -- can be export or import
    #[arg(required = true)]
    command: String,

    ///Path to input ESM/ESP
    #[arg(short, long)]
    input_esm: String,

    ///Path to output ESM/ESP, if importing
    #[arg(short, long)]
    output_esm: String,

    ///Path to image file
    #[arg(long)]
    image: String,
}


const PPC : u32 = 65u32; // Pixels per cell


struct Extents {
    pub min_x : i32,
    pub max_x : i32,
    pub min_y : i32,
    pub max_y : i32,
}


fn calc_plugin_extents(plugin: &Plugin) -> Extents {
    const MAX : i32 = 0x7fffffffi32;
    const MIN : i32 = -0x7fffffffi32;

    let mut e = Extents { min_x: MAX, max_x: MIN, min_y: MAX, max_y: MIN };

    for object in plugin.objects_of_type::<Landscape>() {
        let (x, y) = object.grid; // (i32, i32)
        if x < e.min_x {
            e.min_x = x;
        }
        if x > e.max_x {
            e.max_x = x;
        }
        if y < e.min_y {
            e.min_y = y;
        }
        if y > e.max_y {
            e.max_y = y;
        }
    }

    assert!(e.max_x >= e.min_x);
    assert!(e.max_y >= e.min_y);

    e
}


fn export(args: &Args) -> std::io::Result<()> {

    let plugin = Plugin::from_path(&args.input_esm)?;
    let e = calc_plugin_extents(&plugin);
    let ncells_x:u32 = (e.max_x - e.min_x + 1) as u32;
    let ncells_y:u32 = (e.max_y - e.min_y + 1) as u32;

    let (img_w, img_h) = (ncells_x * PPC, ncells_y * PPC);

    println!("converting {} x {} cells into a {} x {} image", ncells_x, ncells_y, img_w, img_h);

    // Create image
    let mut im = image::RgbImage::new(img_w, img_h);

    // Dump colors
    for object in plugin.objects_of_type::<Landscape>() {
        let (grid_x, grid_y) = object.grid;
        let data = &object.vertex_colors.data;

        for j in 0..(PPC as usize) {
            for i in 0..(PPC as usize) {
                let (r, g, b) = (data[j][i][0], data[j][i][1], data[j][i][2]);
                let pixel_x = ((grid_x - e.min_x) as u32) * PPC + (i as u32);
                let pixel_y = ((grid_y - e.min_y) as u32) * PPC + (j as u32);
                im.put_pixel(pixel_x, (img_h - 1) - pixel_y, image::Rgb([r, g, b]));
            }
        }
    }

    // Save the image
    im.save(&args.image).unwrap();

    Ok(())
}


fn import(args: &Args) -> std::io::Result<()> {

    let mut plugin = Plugin::from_path(&args.input_esm)?;
    let e = calc_plugin_extents(&plugin);
    let ncells_x:u32 = (e.max_x - e.min_x + 1) as u32;
    let ncells_y:u32 = (e.max_y - e.min_y + 1) as u32;

    let im = image::open(&args.image).unwrap().into_rgb8();
    let (img_w, img_h) = im.dimensions();

    assert!(img_w == ncells_x * PPC);
    assert!(img_h == ncells_y * PPC);

    // Read colors
    for object in plugin.objects_of_type_mut::<Landscape>() {
        let (grid_x, grid_y) = object.grid;
        let data = &mut object.vertex_colors.data;

        for j in 0..(PPC as usize) {
            for i in 0..(PPC as usize) {
                let pixel_x = ((grid_x - e.min_x) as u32) * PPC + (i as u32);
                let pixel_y = ((grid_y - e.min_y) as u32) * PPC + (j as u32);
                let pixel = im.get_pixel(pixel_x, (img_h - 1) - pixel_y);
                data[j][i][0] = pixel[0];
                data[j][i][1] = pixel[1];
                data[j][i][2] = pixel[2];
            }
        }
    }

    // Save the ESM/ESP
    plugin.save_path(&args.output_esm)?;

    Ok(())
}


fn main() -> std::io::Result<()> {

    let args = Args::parse();
    match args.command.as_str() {
        "export" => export(&args),
        "import" => import(&args),
        _ => panic!(),
    }?;

    Ok(())
}
