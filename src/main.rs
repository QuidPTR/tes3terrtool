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
    let mut e = Extents {
        min_x: i32::MAX,
        max_x: i32::MIN,
        min_y: i32::MAX,
        max_y: i32::MIN
    };

    for object in plugin.objects_of_type::<Landscape>() {
        let (cell_x, cell_y) = object.grid;
        e.min_x = e.min_x.min(cell_x);
        e.max_x = e.max_x.max(cell_x);
        e.min_y = e.min_y.min(cell_y);
        e.max_y = e.max_y.max(cell_y);
    }

    e
}


fn export(input_esm: &String, output_image: &String, full_dump: bool) {

    let plugin =
        Plugin::from_path(input_esm)
            .expect(format!("ERROR: could not open plugin '{input_esm}'").as_str());

    let e = calc_plugin_extents(&plugin);
    let ncells_x: u32 = (e.max_x - e.min_x + 1) as u32;
    let ncells_y: u32 = (e.max_y - e.min_y + 1) as u32;

    // Pixels per cell
    let ppc = if full_dump { 65 } else { 64 };
    let (img_w, img_h) = (ncells_x * ppc, ncells_y * ppc);
    let mut im = RgbImage::new(img_w, img_h);

    println!("Extracting {ncells_x}x{ncells_y} cells to a {img_w}x{img_w} image");
    println!("Origin cell has coordinates ({}, {})", e.min_x, e.min_y);

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

        //println!("Processing cell ({cell_x}, {cell_y})");

        // Index of the first row/col to be exported
        let first = if full_dump { 0 } else { 1 };

        // NOTE: skips the first row/column, they cannot be edited independently
        for texel_y in first..65 {
            for texel_x in first..65 {
                let (r, g, b) = (
                    data[texel_y][texel_x][0],
                    data[texel_y][texel_x][1],
                    data[texel_y][texel_x][2],
                );
                let pixel_x = ((cell_x - e.min_x) as u32) * ppc + ((texel_x - first) as u32);
                let pixel_y = ((cell_y - e.min_y) as u32) * ppc + ((texel_y - first) as u32);
                im.put_pixel(pixel_x, (img_h - 1) - pixel_y, Rgb([r, g, b]));
            }
        }
    }

    // Save the image
    im.save(output_image).unwrap();
}


fn import(input_esm: &String, input_image: &String, output_esm: &String) {

    let mut plugin =
        Plugin::from_path(input_esm)
            .expect(format!("ERROR: could not open plugin '{input_esm}'").as_str());

    let e = calc_plugin_extents(&plugin);
    let ncells_x: u32 = (e.max_x - e.min_x + 1) as u32;
    let ncells_y: u32 = (e.max_y - e.min_y + 1) as u32;

    let im =
        image::open(input_image)
            .expect(format!("ERROR: could not open image '{input_image}'").as_str())
            .into_rgb8();
    let (img_w, img_h) = im.dimensions();

    let full_dump = if img_w == ncells_x * 64 && img_h == ncells_y * 64 {
        false
    } else if img_w == ncells_x * 65 && img_h == ncells_y * 65 {
        true
    } else {
        panic!("Image size doesn't match plugin size");
    };

    println!("Importing {ncells_x}x{ncells_y} cells from a {img_w}x{img_h} image (full dump: {full_dump})");

    let max_pixel_y: i32 = (img_h - 1) as i32;
    let white = Rgb([0xFFu8, 0xFFu8, 0xFFu8]);

    // Read colors
    for object in plugin.objects_of_type_mut::<Landscape>() {
        let (cell_x, cell_y) = object.grid;
        let data = &mut object.vertex_colors.data;

        //println!("Processing cell ({cell_x}, {cell_y})");

        // Index of the first row/col to be imported
        let ppc = if full_dump { 65 } else { 64 };
        let first = if full_dump { 0 } else { 1 };

        for texel_y in 0..65 {
            for texel_x in 0..65 {
                let pixel_y = (cell_y - e.min_y) * ppc + (texel_y - first);
                let pixel_x = (cell_x - e.min_x) * ppc + (texel_x - first);
                let pixel = im.get_pixel_checked(pixel_x as u32, (max_pixel_y - pixel_y) as u32).unwrap_or(&white);
                data[texel_y as usize][texel_x as usize] = [pixel[0], pixel[1], pixel[2]];
            }
        }

        // TODO: if all white (including borders), don't import

        // Ensure the game knows this cell now has vertex colors
        object.landscape_flags |= LandscapeFlags::USES_VERTEX_COLORS;
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
