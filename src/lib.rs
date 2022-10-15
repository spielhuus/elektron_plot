mod pcb;
mod schema;
mod theme;
mod error;
mod cairo_plotter;
mod border;

use std::fs::File;
use std::io::Write;

pub use self::cairo_plotter::{CairoPlotter, ImageType, PlotItem, Plotter};
pub use self::theme::{Theme, Themer};
use elektron_spice::{Circuit, Netlist};
pub use error::Error;

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
use elektron_sexp::{Schema, Pcb};
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

pub fn plot_schema(schema: &Schema, filename: &str, scale: f64, border: bool, theme: &str, netlist: Option<Netlist>) -> Result<(), Error> {
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

    for i in 0..schema.pages() { //TODO: iterate page directly
        use self::schema::PlotIterator;
        let iter = schema.iter(i)?.plot(schema, &schema.pages[i].title_block, schema.pages[i].paper_size.clone().into(), &theme, border, &netlist).flatten().collect(); //TODO: plot all, remove clone
        let mut cairo = CairoPlotter::new(&iter);
        check_directory(filename)?;
        let out: Box<dyn Write> = Box::new(File::create(filename)?);
        cairo.plot(out, border, scale, &image_type)?;
    }
    Ok(())
}

///plot the pcb.
pub fn plot_pcb(pcb: &Pcb, filename: &str, scale: f64, border: bool, theme: &str) -> Result<(), Error> {
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
