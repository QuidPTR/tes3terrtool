use clap::{Parser, Subcommand};
use image::{Rgb, RgbImage};
use tes3::esp::{Plugin, Landscape, LandscapeFlags};


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

    let (img_w, img_h) = (ncells_x * 64, ncells_y * 64);
    let mut im = RgbImage::new(img_w, img_h);

    println!("Extracting {}x{} cells to a {}x{} image", ncells_x, ncells_y, img_w, img_h);

    // Fill in with white, useful during import
    im.fill(0xFF);

    // Dump colors
    for object in plugin.objects_of_type::<Landscape>() {
        let (cell_x, cell_y) = object.grid;
        let data = &object.vertex_colors.data;

        if !(object.landscape_flags.intersects(LandscapeFlags::USES_VERTEX_COLORS)) {
            // No vertex color to process
            continue;
        }

        println!("Processing cell ({cell_x}, {cell_y})");

        // NOTE: skips the first row/column, they cannot be edited independently
        for texel_y in 1..65 {
            for texel_x in 1..65 {
                let (r, g, b) = (
                    data[texel_y][texel_x][0],
                    data[texel_y][texel_x][1],
                    data[texel_y][texel_x][2],
                );
                let pixel_x = ((cell_x - e.min_x) as u32) * 64 + ((texel_x - 1) as u32);
                let pixel_y = ((cell_y - e.min_y) as u32) * 64 + ((texel_y - 1) as u32);
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

    println!("Importing {}x{} cells from a {}x{} image", ncells_x, ncells_y, img_w, img_h);

    assert!(img_w == ncells_x * 64);
    assert!(img_h == ncells_y * 64);

    // Read colors
    for object in plugin.objects_of_type_mut::<Landscape>() {
        let (cell_x, cell_y) = object.grid;
        let data = &mut object.vertex_colors.data;

        // TODO: if all white (including borders), don't import

        // Ensure the game knows this cell now has vertex colors
        object.landscape_flags |= LandscapeFlags::USES_VERTEX_COLORS;

        println!("Processing cell ({cell_x}, {cell_y})");

        // Take the first pixel from the last pixel of the cell above to the left
        let mut pixel = Rgb([0xFFu8, 0xFFu8, 0xFFu8]);
        if ((cell_x - 1) - e.min_x) > 0 && ((cell_y - 1) - e.min_y) > 0 {
            // There is a cell above to the left
            let pixel_x = (((cell_x - 1) - e.min_x) as u32) * 64 + 63;
            let pixel_y = (((cell_y - 1) - e.min_y) as u32) * 64 + 63;
            pixel = *im.get_pixel(pixel_x, (img_h - 1) - pixel_y);
        }
        data[0][0] = [pixel[0], pixel[1], pixel[2]];

        // Take the first column from the last column of the cell to the left
        for texel_y in 1..65 {
            let mut pixel = Rgb([0xFFu8, 0xFFu8, 0xFFu8]);
            let pixel_y = ((cell_y - e.min_y) as u32) * 64 + ((texel_y - 1) as u32);
            if ((cell_x - 1) - e.min_x) > 0 {
                // There is a cell to the left
                let pixel_x = (((cell_x - 1) - e.min_x) as u32) * 64 + 63;
                pixel = *im.get_pixel(pixel_x, (img_h - 1) - pixel_y);
            }
            data[texel_y][0] = [pixel[0], pixel[1], pixel[2]];
        }

        // Take the first row from the last row of the cell above
        for texel_x in 1..65 {
            let mut pixel = Rgb([0xFFu8, 0xFFu8, 0xFFu8]);
            let pixel_x = ((cell_x - e.min_x) as u32) * 64 + ((texel_x - 1) as u32);
            if ((cell_y - 1) - e.min_y) > 0 {
                // There is a cell above
                let pixel_y = (((cell_y - 1) - e.min_y) as u32) * 64 + 63;
                pixel = *im.get_pixel(pixel_x, (img_h - 1) - pixel_y);
            }
            data[0][texel_x] = [pixel[0], pixel[1], pixel[2]];
        }

        // Copy the rest of the cell
        for texel_y in 1..65 {
            for texel_x in 1..65 {
                let pixel_x = ((cell_x - e.min_x) as u32) * 64 + ((texel_x - 1) as u32);
                let pixel_y = ((cell_y - e.min_y) as u32) * 64 + ((texel_y - 1) as u32);
                let pixel = im.get_pixel(pixel_x, (img_h - 1) - pixel_y);
                data[texel_y][texel_x] = [pixel[0], pixel[1], pixel[2]];
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
