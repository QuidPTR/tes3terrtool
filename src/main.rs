use clap::{Parser, Subcommand};
use image::{Rgb, RgbImage};
use tes3::esp::{Plugin, Landscape, LandscapeFlags, ObjectFlags};


fn calc_extents(grids: &Vec<(i32, i32)>) -> (i32, i32, i32, i32) {
    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;

    for grid in grids {
        let (cell_x, cell_y) = grid;
        min_x = min_x.min(*cell_x);
        max_x = max_x.max(*cell_x);
        min_y = min_y.min(*cell_y);
        max_y = max_y.max(*cell_y);
    }

    (min_x, max_x, min_y, max_y)
}


fn export(input_esm: &String, output_image: &String, full_dump: bool) {

    let plugin =
        Plugin::from_path(input_esm)
            .expect(format!("ERROR: could not open plugin '{input_esm}'").as_str());

    // Retrieve all suitable landscape records
    let landscapes: Vec<_> = {
        plugin.objects_of_type::<Landscape>()
            .filter(|landscape| { 
                if landscape.flags.contains(ObjectFlags::DELETED) {
                    // Don't export deleted cells
                    return false;
                }
                if !landscape
                    .landscape_flags
                    .contains(LandscapeFlags::USES_VERTEX_COLORS) {
                    // Don't export cells without vertex colors
                    return false;
                }
                true
            })
            .collect()
    };

    // Figure out the extents of the plugin
    let cell_coords: Vec<_> = {
        landscapes.iter().map(|landscape| { landscape.grid }).collect()
    };
    let (min_x, max_x, min_y, max_y) = calc_extents(&cell_coords);
    let ncells_x: u32 = (max_x - min_x + 1) as u32;
    let ncells_y: u32 = (max_y - min_y + 1) as u32;

    // Setup pixels per cell and index of the first texel to be exported
    let ppc = if full_dump { 65 } else { 64 };
    let first = if full_dump { 0 } else { 1 };

    // Create a new white image
    let (img_w, img_h) = (ncells_x * ppc, ncells_y * ppc);
    let mut im = RgbImage::new(img_w, img_h);
    im.fill(0xFFu8);

    println!("Extracting {ncells_x}x{ncells_y} cells to a {img_w}x{img_w} image");
    println!("Origin cell has coordinates ({}, {})", min_x, min_y);

    // Export colors
    for landscape in landscapes {
        let (cell_x, cell_y) = landscape.grid;
        let data = &landscape.vertex_colors.data;

        for texel_y in first..65 {
            for texel_x in first..65 {
                let (r, g, b) = (
                    data[texel_y][texel_x][0],
                    data[texel_y][texel_x][1],
                    data[texel_y][texel_x][2],
                );
                let pixel_x = ((cell_x - min_x) as u32) * ppc + ((texel_x - first) as u32);
                let pixel_y = ((cell_y - min_y) as u32) * ppc + ((texel_y - first) as u32);
                im.put_pixel(pixel_x, (img_h - 1) - pixel_y, Rgb([r, g, b]));
            }
        }
    }

    // Save the image
    im.save(output_image).unwrap();
}


fn is_full_dump(img_w: u32, img_h: u32, ncells_x: u32, ncells_y: u32) -> bool {
    if img_w == ncells_x * 64 && img_h == ncells_y * 64 {
        return false;
    } else if img_w == ncells_x * 65 && img_h == ncells_y * 65 {
        return true;
    } else {
        panic!("Image size doesn't match plugin size");
    };
}


fn import(input_esm: &String, input_image: &String, output_esm: &String) {

    let mut plugin =
        Plugin::from_path(input_esm)
            .expect(format!("ERROR: could not open plugin '{input_esm}'").as_str());

    // Retrieve all suitable landscape records
    let landscapes: Vec<_> = {
        plugin.objects_of_type_mut::<Landscape>()
            .filter(|landscape| { 
                if landscape.flags.contains(ObjectFlags::DELETED) {
                    // Don't import deleted cells
                    return false;
                }
                if !landscape
                    .landscape_flags
                    .contains(LandscapeFlags::USES_VERTEX_COLORS) {
                    // Don't import cells without vertex colors
                    return false;
                }
                true
            })
            .collect()
    };

    // Figure out the extents of the plugin
    let cell_coords: Vec<_> = {
        landscapes.iter().map(|landscape| { landscape.grid }).collect()
    };
    let (min_x, max_x, min_y, max_y) = calc_extents(&cell_coords);
    let ncells_x: u32 = (max_x - min_x + 1) as u32;
    let ncells_y: u32 = (max_y - min_y + 1) as u32;

    let im =
        image::open(input_image)
            .expect(format!("ERROR: could not open image '{input_image}'").as_str())
            .into_rgb8();
    let (img_w, img_h) = im.dimensions();
    let full_dump: bool = is_full_dump(img_w, img_h, ncells_x, ncells_y);

    println!("Importing {ncells_x}x{ncells_y} cells from a {img_w}x{img_h} image (full dump: {full_dump})");

    // Setup pixels per cell and index of the first texel to be imported
    let ppc = if full_dump { 65 } else { 64 };
    let first = if full_dump { 0 } else { 1 };
    let max_pixel_y: i32 = (img_h - 1) as i32;
    let white = Rgb([0xFFu8, 0xFFu8, 0xFFu8]);

    // Import colors
    for landscape in landscapes {
        let (cell_x, cell_y) = landscape.grid;
        let data = &mut landscape.vertex_colors.data;

        for texel_y in 0..65 {
            for texel_x in 0..65 {
                let pixel_y = (cell_y - min_y) * ppc + (texel_y - first);
                let pixel_x = (cell_x - min_x) * ppc + (texel_x - first);
                let pixel = im.get_pixel_checked(pixel_x as u32, (max_pixel_y - pixel_y) as u32).unwrap_or(&white);
                data[texel_y as usize][texel_x as usize] = [pixel[0], pixel[1], pixel[2]];
            }
        }

    }

    // Save the ESM/ESP
    plugin.save_path(output_esm).unwrap();
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

        #[arg(long, short, action)]
        /// Export the entire 65x65 vertex color block, rather than a 64x64 slice
        full_dump: bool,
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
        Some(Commands::ExportVcol { input_esm, output_image, full_dump }) => {
            println!("Exporting vertex color data: {input_esm} -> {output_image}");
            export(&input_esm, &output_image, *full_dump);
        },
        Some(Commands::ImportVcol { input_esm, input_image, output_esm }) => {
            println!("Importing vertex color data: {input_esm} + {input_image} -> {output_esm}");
            import(&input_esm, &input_image, &output_esm);
        },
        None => {
            println!("Invalid command, try import-vcol or export-vcol");
        }
    };
}
