use clap::{Parser, Subcommand};
use image::{Rgb, RgbImage};
use tes3::esp::{Plugin, Landscape};


const PPC : u32 = 65u32; // Pixels per cell


struct Extents {
    pub min_x: i32,
    pub max_x: i32,
    pub min_y: i32,
    pub max_y: i32,
}


fn calc_plugin_extents(plugin: &Plugin) -> Extents {
    const MAX: i32 = 0x7fffffffi32;
    const MIN: i32 = -0x7fffffffi32;

    let mut e = Extents { min_x: MAX, max_x: MIN, min_y: MAX, max_y: MIN };

    for object in plugin.objects_of_type::<Landscape>() {
        let (cell_x, cell_y) = object.grid; // (i32, i32)
        if cell_x < e.min_x {
            e.min_x = cell_x;
        }
        if cell_x > e.max_x {
            e.max_x = cell_x;
        }
        if cell_y < e.min_y {
            e.min_y = cell_y;
        }
        if cell_y > e.max_y {
            e.max_y = cell_y;
        }
    }

    assert!(e.max_x >= e.min_x);
    assert!(e.max_y >= e.min_y);

    e
}


fn export(input_esm: &String, output_image: &String) -> std::io::Result<()> {

    let plugin = Plugin::from_path(input_esm)?;
    let e = calc_plugin_extents(&plugin);
    let ncells_x: u32 = (e.max_x - e.min_x + 1) as u32;
    let ncells_y: u32 = (e.max_y - e.min_y + 1) as u32;

    let (img_w, img_h) = (ncells_x * PPC, ncells_y * PPC);

    println!("converting {} x {} cells into a {} x {} image", ncells_x, ncells_y, img_w, img_h);

    // Create image
    let mut im = RgbImage::new(img_w, img_h);

    // Dump colors
    for object in plugin.objects_of_type::<Landscape>() {
        let (cell_x, cell_y) = object.grid;
        let data = &object.vertex_colors.data;

        for j in 0..(PPC as usize) {
            for i in 0..(PPC as usize) {
                let (r, g, b) = (data[j][i][0], data[j][i][1], data[j][i][2]);
                let pixel_x = ((cell_x - e.min_x) as u32) * PPC + (i as u32);
                let pixel_y = ((cell_y - e.min_y) as u32) * PPC + (j as u32);
                im.put_pixel(pixel_x, (img_h - 1) - pixel_y, Rgb([r, g, b]));
            }
        }
    }

    // Save the image
    im.save(output_image).unwrap();

    Ok(())
}


fn import(input_esm: &String, input_image: &String, output_esm: &String) -> std::io::Result<()> {

    let mut plugin = Plugin::from_path(input_esm)?;
    let e = calc_plugin_extents(&plugin);
    let ncells_x: u32 = (e.max_x - e.min_x + 1) as u32;
    let ncells_y: u32 = (e.max_y - e.min_y + 1) as u32;

    let im = image::open(input_image).unwrap().into_rgb8();
    let (img_w, img_h) = im.dimensions();

    assert!(img_w == ncells_x * PPC);
    assert!(img_h == ncells_y * PPC);

    // Read colors
    for object in plugin.objects_of_type_mut::<Landscape>() {
        let (cell_x, cell_y) = object.grid;
        let data = &mut object.vertex_colors.data;

        for j in 0..(PPC as usize) {
            for i in 0..(PPC as usize) {
                let pixel_x = ((cell_x - e.min_x) as u32) * PPC + (i as u32);
                let pixel_y = ((cell_y - e.min_y) as u32) * PPC + (j as u32);
                let pixel = im.get_pixel(pixel_x, (img_h - 1) - pixel_y);
                data[j][i][0] = pixel[0];
                data[j][i][1] = pixel[1];
                data[j][i][2] = pixel[2];
            }
        }
    }

    // Save the ESM/ESP
    plugin.save_path(output_esm)?;

    Ok(())
}


#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {

    #[command(subcommand)]
    command: Option<Commands>,
}


#[derive(Subcommand)]
enum Commands {

    /// Exports vertex color data from an ESM into an image
    ExportVcol {
        #[arg(long)]
        /// Path to source ESM
        input_esm: String,

        #[arg(long)]
        /// Path to image that will hold the vertex color data
        output_image: String,
    },

    /// Imports vertex color data from an image into an ESM
    ImportVcol {
        #[arg(long)]
        /// Path to source ESM
        input_esm: String,

        #[arg(long)]
        /// Path to image that holds the vertex color data
        input_image: String,

        #[arg(long)]
        /// Path to output ESM, will overwrite existing files
        output_esm: String,
    },
}


fn main() {
    let args = Args::parse();

    match &args.command {
        Some(Commands::ExportVcol { input_esm, output_image }) => {
            println!("Exporting vertex color data: {input_esm} -> {output_image}");
            let _ = export(&input_esm, &output_image);
        },
        Some(Commands::ImportVcol { input_esm, input_image, output_esm }) => {
            println!("Importing vertex color data: {input_esm} + {input_image} -> {output_esm}");
            let _ = import(&input_esm, &input_image, &output_esm);
        },
        None => {
            println!("Invalid command, try import-vcol or export-vcol");
        }
    };
}
