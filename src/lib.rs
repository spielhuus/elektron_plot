mod border;
mod cairo_plotter;
mod error;
mod pcb;
mod schema;
mod theme;

use lazy_static::lazy_static;
use rand::Rng;
use std::env::temp_dir;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::sync::Mutex;

pub use self::cairo_plotter::{CairoPlotter, ImageType, PlotItem, Plotter};
pub use self::theme::{Theme, Themer};
use elektron_spice::{Circuit, Netlist};
pub use error::Error;

lazy_static! {
    static ref PLOT: Mutex<Vec<Vec<u8>>> = Mutex::new(vec![]);
}

pub fn store_plot(plot: Vec<u8>) {
    PLOT.lock().unwrap().push(plot);
}

pub fn get_plots() -> Vec<Vec<u8>> {
    let mut res = Vec::new();
    PLOT.lock()
        .unwrap()
        .iter()
        .for_each(|i| res.push(i.clone()));
    res
}

pub fn reset_plots() {
    PLOT.lock().unwrap().clear()
}

macro_rules! text {
    ($pos:expr, $angle:expr, $content:expr, $effects:expr) => {
        PlotItem::Text(
            99,
            Text::new(
                $pos,
                $angle,
                $content,
                $effects.color.clone(),
                $effects.font_size.0,
                $effects.font.as_str(),
                $effects.justify.clone(),
                false,
            ),
        )
    };
}
use elektron_sexp::{Pcb, Schema};
pub(crate) use text;

fn check_directory(filename: &str) -> Result<(), Error> {
    let path = std::path::Path::new(filename);
    let parent = path.parent();
    if let Some(parent) = parent {
        if parent.to_str().unwrap() != "" && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

pub fn plot_schema(
    schema: &Schema,
    filename: Option<&str>,
    scale: f64,
    border: bool,
    theme: &str,
    netlist: Option<Netlist>,
    image_type: Option<&str>,
) -> Result<(), Error> {
    let image_type = if let Some(image_type) = image_type {
        if image_type == "pdf" {
            Ok(ImageType::Pdf)
        } else if image_type == "png" {
            Ok(ImageType::Png)
        } else if image_type == "svg" {
            Ok(ImageType::Svg)
        } else {
            Err(Error::UnknownImageType(image_type.to_string()))
        }
    } else {
        Ok(ImageType::Svg)
    }?;
    let theme = if theme == "mono" {
        Theme::mono()
    } else {
        Theme::kicad_2000()
    };
    if let Some(filename) = filename {
        for i in 0..schema.pages() {
            //TODO: iterate page directly
            use self::schema::PlotIterator;
            let iter = schema
                .iter(i)?
                .plot(
                    schema,
                    &schema.pages[i].title_block,
                    schema.pages[i].paper_size.clone().into(),
                    &theme,
                    border,
                    &netlist,
                )
                .flatten()
                .collect(); //TODO: plot all, remove clone
            let mut cairo = CairoPlotter::new(&iter);
            check_directory(filename)?;
            let out: Box<dyn Write> = Box::new(File::create(filename)?);
            cairo.plot(out, border, scale, &image_type)?;
        }
    } else {
        for i in 0..schema.pages() {
            //TODO: iterate page directly
            let mut rng = rand::thread_rng();
            let num: u32 = rng.gen();
            let filename = String::new() + temp_dir().to_str().unwrap() + "/" + &num.to_string(); //TODO: add
                                                                                                  //extension

            use self::schema::PlotIterator;
            let iter = schema
                .iter(i)?
                .plot(
                    schema,
                    &schema.pages[i].title_block,
                    schema.pages[i].paper_size.clone().into(),
                    &theme,
                    border,
                    &netlist,
                )
                .flatten()
                .collect(); //TODO: plot all, remove clone
            let mut cairo = CairoPlotter::new(&iter);
            let out: Box<dyn Write> = Box::new(File::create(&filename)?);
            cairo.plot(out, border, scale, &image_type)?;

            let mut f = File::open(&filename).expect("no file found");
            let metadata = fs::metadata(&filename).expect("unable to read metadata");
            let mut buffer = vec![0; metadata.len() as usize];
            f.read_exact(&mut buffer).expect("buffer overflow");
            store_plot(buffer);
        }
    }
    Ok(())
}

pub fn plot_schema_buffer(
    schema: &Schema,
    callback: &dyn Fn(Vec<u8>),
    scale: f64,
    border: bool,
    theme: &str,
    netlist: Option<Netlist>,
    image_type: &str,
) -> Result<(), Error> {
    let image_type = if image_type == "pdf" {
        Ok(ImageType::Pdf)
    } else if image_type == "png" {
        Ok(ImageType::Png)
    } else if image_type == "svg" {
        Ok(ImageType::Svg)
    } else {
        Err(Error::UnknownImageType(image_type.to_string()))
    }?;

    let theme = if theme == "mono" {
        Theme::mono()
    } else {
        Theme::kicad_2000()
    };

    for i in 0..schema.pages() {
        //TODO: iterate page directly
        let mut rng = rand::thread_rng();
        let num: u32 = rng.gen();
        let filename = String::new() + temp_dir().to_str().unwrap() + "/" + &num.to_string(); //TODO: add
                                                                                              //extension

        use self::schema::PlotIterator;
        let iter = schema
            .iter(i)?
            .plot(
                schema,
                &schema.pages[i].title_block,
                schema.pages[i].paper_size.clone().into(),
                &theme,
                border,
                &netlist,
            )
            .flatten()
            .collect(); //TODO: plot all, remove clone
        let mut cairo = CairoPlotter::new(&iter);
        let out: Box<dyn Write> = Box::new(File::create(&filename)?);
        cairo.plot(out, border, scale, &image_type)?;

        let mut f = File::open(&filename).expect("no file found");
        let metadata = fs::metadata(&filename).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read_exact(&mut buffer).expect("buffer overflow");
        callback(buffer);
    }
    Ok(())
}
///plot the pcb.
pub fn plot_pcb(
    pcb: &Pcb,
    filename: &str,
    scale: f64,
    border: bool,
    theme: &str,
) -> Result<(), Error> {
    let image_type = if filename.ends_with(".svg") {
        ImageType::Svg
    } else if filename.ends_with(".png") {
        ImageType::Png
    } else {
        ImageType::Pdf
    };
    let theme = if theme == "mono" {
        Theme::mono()
    } else {
        Theme::kicad_2000()
    };

    use self::pcb::PcbPlotIterator;
    let iter = pcb.iter()?.plot(pcb, theme, border).flatten().collect();
    let mut cairo = CairoPlotter::new(&iter); //TODO: set title block

    check_directory(filename)?;
    let out: Box<dyn Write> = Box::new(File::create(filename)?);
    cairo.plot(out, border, scale, &image_type)?;
    Ok(())
}
