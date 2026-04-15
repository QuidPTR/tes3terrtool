use clap::Parser;
use tes3::esp::{Plugin, Landscape};
use image::{RgbImage, Rgb};


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    ///Path to ESM/ESP file
    #[arg(short, long)]
    esm: String,

    ///Path to BMP file
    #[arg(short, long)]
    image: String,
}


fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let plugin = Plugin::from_path(args.esm)?;

    const PPC : u32 = 65u32; // Pixels per cell

    // Figure out extents of the ESM 
    let (mut min_x, mut max_x) = (0x7fffffff, -0x7fffffff);
    let (mut min_y, mut max_y) = (0x7fffffff, -0x7fffffff);
    for object in plugin.objects_of_type::<Landscape>() {
        let (x, y) = object.grid;
        if x < min_x {
            min_x = x;
        }
        if x > max_x {
            max_x = x;
        }
        if y < min_y {
            min_y = y;
        }
        if y > max_y {
            max_y = y;
        }
    }
    assert!(max_x >= min_x);
    assert!(max_y >= min_y);

    let dx:u32 = (max_x - min_x + 1) as u32;
    let dy:u32 = (max_y - min_y + 1) as u32;

    // Create an image
    let (img_w, img_h) = (dx * PPC, dy * PPC);
    println!("converting {} x {} cells into a {} x {} image", dx, dy, img_w, img_h);

    let mut im = image::RgbImage::new(img_w, img_h);

    // Dump colors
    for object in plugin.objects_of_type::<Landscape>() {
        let (grid_x, grid_y) = object.grid;
        let data = &object.vertex_colors.data;

        for j in 0..(PPC as usize) {
            for i in 0..(PPC as usize) {
                let (r, g, b) = (data[j][i][0], data[j][i][1], data[j][i][2]);
                let pixel_x = ((grid_x - min_x) as u32) * PPC + (i as u32);
                let pixel_y = ((grid_y - min_y) as u32) * PPC + (j as u32);
                im.put_pixel(pixel_x, (img_h - 1) - pixel_y, Rgb([r, g, b]));
            }
        }
    }

    // Save the image
    im.save(args.image).unwrap();

    Ok(())
}
