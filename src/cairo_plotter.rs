use crate::error::Error;
use itertools::Itertools;
use ndarray::{arr1, arr2, s, Array1, Array2};
use pangocairo::{create_layout, pango::SCALE, show_layout, update_layout};
use std::io::Write;
extern crate cairo;
use cairo::{Context, FontFace, FontSlant, FontWeight, Format, ImageSurface, SvgSurface};

fn rgba_color(color: (f64, f64, f64, f64)) -> String {
    format!("#{:02X}{:02X}{:02X}{:02X}", (color.0*255.0) as u32, (color.1*255.0) as u32, (color.2*255.0) as u32, (color.3*255.0) as u32)
}

pub mod paper {
    pub const A4: (f64, f64) = (297.0, 210.0);
}

pub enum Surface {
    ImageSurface(ImageSurface),
    SvgSurface(SvgSurface),
    // PdfSurface(PdfSurface), TODO
}

pub enum ImageType {
    Svg,
    Png,
    Pdf,
}

#[derive(Debug)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

#[derive(Debug)]
pub struct Line {
    pub pts: Array2<f64>,
    pub linewidth: f64,
    pub linetype: String,
    pub linecap: LineCap,
    pub color: (f64, f64, f64, f64),
}
impl Line {
    pub fn new(
        pts: Array2<f64>,
        linewidth: f64,
        linetype: String,
        linecap: LineCap,
        color: (f64, f64, f64, f64),
    ) -> Line {
        Line {
            pts,
            linewidth,
            linetype,
            linecap,
            color,
        }
    }
}

#[derive(Debug)]
pub struct Rectangle {
    pub pts: Array2<f64>,
    pub color: (f64, f64, f64, f64),
    pub linewidth: f64,
    pub linetype: String,
    pub fill: Option<(f64, f64, f64, f64)>,
}
impl Rectangle {
    pub fn new(
        pts: Array2<f64>,
        color: (f64, f64, f64, f64),
        linewidth: f64,
        linetype: String,
        fill: Option<(f64, f64, f64, f64)>,
    ) -> Rectangle {
        Rectangle {
            pts,
            color,
            linewidth,
            linetype,
            fill,
        }
    }
}
#[derive(Debug)]
pub struct Arc {
    pub start: Array1<f64>,
    pub mid: Array1<f64>,
    pub end: Array1<f64>,
    pub linewidth: f64,
    pub linetype: String,
    pub color: (f64, f64, f64, f64),
    pub fill: Option<(f64, f64, f64, f64)>,
}
impl Arc {
    pub fn new(
        start: Array1<f64>,
        mid: Array1<f64>,
        end: Array1<f64>,
        linewidth: f64,
        linetype: String,
        color: (f64, f64, f64, f64),
        fill: Option<(f64, f64, f64, f64)>,
    ) -> Arc {
        Arc {
            start,
            mid,
            end,
            linewidth,
            linetype,
            color,
            fill,
        }
    }
}
#[derive(Debug)]
pub struct Circle {
    pub pos: Array1<f64>,
    pub radius: f64,
    pub linewidth: f64,
    pub linetype: String,
    pub color: (f64, f64, f64, f64),
    pub fill: Option<(f64, f64, f64, f64)>,
}
impl Circle {
    pub fn new(
        pos: Array1<f64>,
        radius: f64,
        linewidth: f64,
        linetype: String,
        color: (f64, f64, f64, f64),
        fill: Option<(f64, f64, f64, f64)>,
    ) -> Circle {
        Circle {
            pos,
            radius,
            linewidth,
            linetype,
            color,
            fill,
        }
    }
}
#[derive(Debug)]
pub struct Polyline {
    pub pts: Array2<f64>,
    pub color: (f64, f64, f64, f64),
    pub linewidth: f64,
    pub linetype: String,
    pub fill: Option<(f64, f64, f64, f64)>,
}
impl Polyline {
    pub fn new(
        pts: Array2<f64>,
        color: (f64, f64, f64, f64),
        linewidth: f64,
        linetype: String,
        fill: Option<(f64, f64, f64, f64)>,
    ) -> Polyline {
        Polyline {
            pts,
            color,
            linewidth,
            linetype,
            fill,
        }
    }
}

#[derive(Debug)]
pub struct Text {
    pub pos: Array1<f64>,
    pub text: String,
    pub color: (f64, f64, f64, f64),
    pub fontsize: f64,
    pub font: String,
    pub align: Vec<String>,
    pub angle: f64,
    pub label: bool,
}
impl Text {
    pub fn new(
        pos: Array1<f64>,
        angle: f64,
        text: String,
        color: (f64, f64, f64, f64),
        fontsize: f64,
        font: &str,
        align: Vec<String>,
        label: bool,
    ) -> Text {
        Text {
            pos,
            text,
            color,
            fontsize,
            font: font.to_string(),
            align,
            angle,
            label,
        }
    }
}

#[derive(Debug)]
pub enum PlotItem {
    Arc(usize, Arc),
    Circle(usize, Circle),
    Line(usize, Line),
    Rectangle(usize, Rectangle),
    Polyline(usize, Polyline),
    Text(usize, Text),
}

macro_rules! stroke {
    ($context:expr, $stroke:expr) => {
        $context.set_source_rgba(
            $stroke.color.0,
            $stroke.color.1,
            $stroke.color.2,
            $stroke.color.3,
        );
        $context.set_line_width($stroke.linewidth);
    };
}
macro_rules! fill {
    ($context:expr, $fill:expr) => {
        if let Some(fill) = $fill {
            $context.set_source_rgba(fill.0, fill.1, fill.2, fill.3);
            $context.fill().unwrap();
        }
    };
}
macro_rules! effects {
    ($context:expr, $effects:expr) => {
        $context.set_font_size($effects.fontsize);
        let face = FontFace::toy_create(
            $effects.font.as_str(),
            FontSlant::Normal,
            FontWeight::Normal,
        )
        .unwrap();
        $context.set_font_face(&face);
        $context.set_source_rgba(0.0, 0.0, 0.0, 1.0); //TODO
    };
}

pub trait Plotter {
    fn text_size(&self, item: &Text) -> Array1<f64>;
    fn bounds(&self) -> Array2<f64>;
    fn plot(
        &mut self,
        file: Box<dyn Write>,
        border: bool,
        scale: f64,
        image_type: &ImageType,
    ) -> Result<(), Error>;
    fn paper(&mut self, paper: String);
    fn get_paper(&self) -> (f64, f64);
    fn draw(&mut self, context: &Context);
}

/// Plotter implemntation for SVG and PDF file.
pub struct CairoPlotter<'a> {
    items: &'a Vec<PlotItem>,
    context: Context,
    paper_size: (f64, f64),
}
impl<'a> CairoPlotter<'a> {
    pub fn new(items: &'a Vec<PlotItem>) -> CairoPlotter {
        let surface = ImageSurface::create(
            Format::Rgb24,
            (297.0 * 72.0 / 25.4) as i32,
            (210.0 * 72.0 / 25.4) as i32,
        )
        .unwrap();
        let context = Context::new(&surface).unwrap();
        context.scale(72.0 / 25.4, 72.0 / 25.4);
        CairoPlotter {
            items,
            context,
            paper_size: paper::A4,
        }
    }
    fn arr_outline(&self, boxes: &Array2<f64>) -> Array2<f64> {
        let axis1 = boxes.slice(s![.., 0]);
        let axis2 = boxes.slice(s![.., 1]);
        arr2(&[
            [
                *axis1
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
                *axis2
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
            ],
            [
                *axis1
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
                *axis2
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
            ],
        ])
    }
}

impl<'a> Plotter for CairoPlotter<'a> {
    /// get the text size in pixels.
    fn text_size(&self, item: &Text) -> Array1<f64> {
        effects!(self.context, item);
        let extends = self.context.text_extents(item.text.as_str()).unwrap();
        arr1(&[extends.width, extends.height])
    }
    /// Calculate the drawing area.
    fn bounds(&self) -> Array2<f64> {
        let mut __bounds: Array2<f64> = Array2::default((0, 2));
        self.items.iter().for_each(|item| {
            let arr: Option<Array2<f64>> = match item {
                PlotItem::Arc(_, arc) => Option::from(arr2(&[
                    [arc.start[0], arc.start[1]],
                    [arc.end[0], arc.end[1]],
                ])),
                PlotItem::Line(_, line) => Option::from(arr2(&[
                    [line.pts[[0, 0]], line.pts[[0, 1]]],
                    [line.pts[[1, 0]], line.pts[[1, 1]]],
                ])),
                PlotItem::Text(_, text) => {
                    let outline = self.text_size(text);
                    let mut x = text.pos[0];
                    let mut y = text.pos[1];
                    if text.align.contains(&String::from("right")) {
                        x -= outline[0];
                    } else if text.align.contains(&String::from("top")) {
                        y -= outline[1];
                    } else if !text.align.contains(&String::from("left"))
                        && !text.align.contains(&String::from("bottom"))
                    {
                        x -= outline[0] / 2.0;
                        y -= outline[1] / 2.0;
                    }
                    Option::from(arr2(&[[x, y], [x + outline[0], y + outline[1]]]))
                }
                PlotItem::Circle(_, circle) => Option::from(arr2(&[
                    [circle.pos[0] - circle.radius, circle.pos[1] - circle.radius],
                    [circle.pos[0] + circle.radius, circle.pos[1] + circle.radius],
                ])),
                PlotItem::Polyline(_, polyline) => Option::from(self.arr_outline(&polyline.pts)),
                PlotItem::Rectangle(_, rect) => Option::from(arr2(&[
                    [rect.pts[[0, 0]], rect.pts[[0, 1]]],
                    [rect.pts[[1, 0]], rect.pts[[1, 1]]],
                ])),
            };
            if let Some(array) = arr {
                for row in array.rows() {
                    __bounds.push_row(row).unwrap();
                }
            }
        });
        self.arr_outline(&__bounds)
    }

    fn plot(
        &mut self,
        mut file: Box<dyn Write>,
        border: bool,
        scale: f64,
        image_type: &ImageType,
    ) -> Result<(), Error> {
        if border {
            match image_type {
                ImageType::Svg => {
                    let surface = SvgSurface::for_stream(
                        self.paper_size.0 * 96.0 / 25.4,
                        self.paper_size.1 * 96.0 / 25.4,
                        file,
                    )
                    .unwrap();
                    let context = Context::new(&surface).unwrap();
                    context.scale(96.0 / 25.4, 96.0 / 25.4);
                    self.draw(&context);
                    surface.finish_output_stream().unwrap();
                }
                ImageType::Png => {
                    let surface = ImageSurface::create(
                        Format::Rgb24,
                        (self.paper_size.0 * 96.0 / 25.4) as i32,
                        (self.paper_size.1 * 96.0 / 25.4) as i32,
                    )
                    .unwrap();
                    let context = Context::new(&surface).unwrap();
                    context.scale(96.0 / 25.4, 96.0 / 25.4);
                    self.draw(&context);
                    surface.write_to_png(&mut file)?;
                }
                ImageType::Pdf => {
                    todo!();
                    /*    let surface = PdfSurface::for_stream(
                        self.paper_size.0 * 96.0 / 25.4,
                        self.paper_size.1 * 96.0 / 25.4,
                        file,
                    )
                    .unwrap();
                    let context = Context::new(&surface).unwrap();
                    context.scale(96.0 / 25.4, 96.0 / 25.4);
                    (context, Surface::PdfSurface(surface)) */
                }
            }
        } else {
            match image_type {
                ImageType::Svg => {
                    let size = self.bounds() + arr2(&[[-2.54, -2.54], [2.54, 2.54]]);
                    let surface = SvgSurface::for_stream(
                        (size[[1, 0]] - size[[0, 0]]) * 72.0 / 25.4 * scale,
                        (size[[1, 1]] - size[[0, 1]]) * 72.0 / 25.4 * scale,
                        file,
                    )?;
                    let context = Context::new(&surface)?;
                    context.scale(72.0 / 25.4 * scale, 72.0 / 25.4 * scale);
                    context.translate(-size[[0, 0]], -size[[0, 1]]);
                    self.draw(&context);
                    surface.finish_output_stream().unwrap();
                }
                ImageType::Png => {
                    let size = self.bounds() + arr2(&[[-2.54, -2.54], [2.54, 2.54]]);
                    let surface = ImageSurface::create(
                        Format::Rgb24,
                        ((size[[1, 0]] - size[[0, 0]]) * 72.0 / 25.4 * scale) as i32,
                        ((size[[1, 1]] - size[[0, 1]]) * 72.0 / 25.4 * scale) as i32,
                    )?;
                    let context = Context::new(&surface)?;
                    context.scale(72.0 / 25.4 * scale, 72.0 / 25.4 * scale);
                    context.translate(-size[[0, 0]], -size[[0, 1]]);
                    self.draw(&context);
                    surface.write_to_png(&mut file)?;
                }
                ImageType::Pdf => {
                    todo!();
                }
            }
        };
        Ok(())
    }

    fn paper(&mut self, paper_size: String) {
        if paper_size == "A4" {
            self.paper_size = paper::A4;
        } // TODO other paper sizes
    }
    fn get_paper(&self) -> (f64, f64) {
        self.paper_size
    }

    fn draw(&mut self, context: &Context) {
        context.set_source_rgb(1.0, 1.0, 1.0);
        context.paint().unwrap();

        //draw the rest
        self.items
            .iter()
            .sorted_by(|a, b| {
                let za = match a {
                    PlotItem::Arc(z, _) => z,
                    PlotItem::Line(z, _) => z,
                    PlotItem::Text(z, _) => z,
                    PlotItem::Circle(z, _) => z,
                    PlotItem::Polyline(z, _) => z,
                    PlotItem::Rectangle(z, _) => z,
                };
                let zb = match b {
                    PlotItem::Arc(z, _) => z,
                    PlotItem::Line(z, _) => z,
                    PlotItem::Text(z, _) => z,
                    PlotItem::Circle(z, _) => z,
                    PlotItem::Polyline(z, _) => z,
                    PlotItem::Rectangle(z, _) => z,
                };

                Ord::cmp(&za, &zb)
            })
            .for_each(|item| {
                //for item in &self.items {
                match item {
                    PlotItem::Line(_, line) => {
                        stroke!(context, line);
                        match line.linecap {
                            LineCap::Butt => context.set_line_cap(cairo::LineCap::Butt),
                            LineCap::Round => context.set_line_cap(cairo::LineCap::Round),
                            LineCap::Square => context.set_line_cap(cairo::LineCap::Square),
                        }
                        context.move_to(line.pts[[0, 0]], line.pts[[0, 1]]);
                        context.line_to(line.pts[[1, 0]], line.pts[[1, 1]]);
                        context.stroke().unwrap();
                    }
                    PlotItem::Polyline(_, line) => {
                        stroke!(context, line);
                        let mut first: bool = true;
                        for pos in line.pts.rows() {
                            if first {
                                context.move_to(pos[0], pos[1]);
                                first = false;
                            } else {
                                context.line_to(pos[0], pos[1]);
                                context.stroke_preserve().unwrap();
                            }
                        }
                        fill!(context, &line.fill);
                        context.stroke().unwrap()
                    }
                    PlotItem::Rectangle(_, rectangle) => {
                        stroke!(context, rectangle);
                        context.rectangle(
                            rectangle.pts[[0, 0]],
                            rectangle.pts[[0, 1]],
                            rectangle.pts[[1, 0]] - rectangle.pts[[0, 0]],
                            rectangle.pts[[1, 1]] - rectangle.pts[[0, 1]],
                        );
                        context.stroke_preserve().unwrap();
                        fill!(context, &rectangle.fill);
                        context.stroke().unwrap()
                    }
                    PlotItem::Circle(_, circle) => {
                        stroke!(context, circle);
                        context.arc(circle.pos[0], circle.pos[1], circle.radius, 0., 10.);
                        context.stroke_preserve().unwrap();
                        fill!(context, &circle.fill);
                        context.stroke().unwrap()
                    }
                    PlotItem::Arc(_, arc) => {
                        stroke!(context, arc);
                        context.arc(arc.start[0], arc.start[1], arc.mid[1], 0., 10.);
                        context.stroke_preserve().unwrap();
                        fill!(context, &arc.fill);
                        context.stroke().unwrap()
                    }
                    PlotItem::Text(_, text) => {
                        context.save().unwrap();
                        let layout = create_layout(&self.context);
                        if let Some(layout) = layout {
                            let markup = format!("<span face=\"{}\" foreground=\"{}\" size=\"{}\">{}</span>", 
                                text.font, rgba_color(text.color), (text.fontsize * 1024.0) as i32, text.text);
                            layout.set_markup(markup.as_str());
                            update_layout(context, &layout);

                            let outline: (i32, i32) = layout.size();
                            let outline = (outline.0 as f64 / SCALE as f64, outline.1 as f64 / SCALE as f64);
                            let mut x = text.pos[0];
                            let mut y = text.pos[1];

                            if !text.label {
                                if text.angle == 0.0 || text.angle == 180.0 {
                                    if text.align.contains(&String::from("right")) {
                                        x -= outline.0 as f64;
                                    } else if !text.align.contains(&String::from("left")) {
                                        x -= outline.0 as f64 / 2.0;
                                    }
                                    if text.align.contains(&String::from("bottom")) {
                                        y -= outline.1 as f64;
                                    } else if !text.align.contains(&String::from("top")) {
                                        y -= outline.1 as f64 / 2.0;
                                    }
                                } else if text.angle == 90.0 || text.angle == 270.0 {
                                    if text.align.contains(&String::from("right")) {
                                        y += outline.0 as f64;
                                    } else if !text.align.contains(&String::from("left")) {
                                        y += outline.0 as f64 / 2.0;
                                    }
                                    if text.align.contains(&String::from("bottom")) {
                                        x -= outline.1 as f64;
                                    } else if !text.align.contains(&String::from("top")) {
                                        x -= outline.1 as f64 / 2.0;
                                    }
                                } else {
                                    println!("text angle is: {} ({})", text.angle, text.text);
                                }
                                context.move_to(x, y);
                                let angle = if text.angle >= 180.0 {
                                    text.angle - 180.0 
                                } else { text.angle };
                                context.rotate(-angle * std::f64::consts::PI / 180.0);
                                show_layout(context, &layout);
                                context.stroke().unwrap();
                            } else {
                                let label_left = 0.4;
                                let label_up = 0.1;
                                let contur = arr2(&[
                                    [0.0, 0.], 
                                    [2.0*label_left, -outline.1/2.0 - label_up],
                                    [3.0*label_left + outline.0, -outline.1/2.0 - label_up],
                                    [3.0*label_left + outline.0, outline.1/2.0 + label_up],
                                    [2.0*label_left, outline.1/2.0 + label_up],
                                    [0.0, 0.0]
                                ]);
                                let theta = -text.angle.to_radians();
                                let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
                                let verts: Array2<f64> = contur.dot(&rot);
                                let verts = &text.pos + verts;
                                context.move_to(text.pos[0], text.pos[1]);
                                for row in verts.rows() {
                                    context.line_to(row[0], row[1]);
                                }
                                context.stroke().unwrap();

                                //adjust the text
                                if text.angle == 0.0 {
                                    x += 2.0*label_left;
                                    y -= outline.1 / 2.0;
                                } else if text.angle == 180.0 {
                                    x -= 2.0*label_left + outline.0;
                                    y -= outline.1 / 2.0;
                                } //TODO 90, 270
                                context.move_to(x, y);
                                let angle = if text.angle >= 180.0 {
                                    text.angle - 180.0 
                                } else { text.angle };
                                context.rotate(-angle * std::f64::consts::PI / 180.0);
                                show_layout(context, &layout);
                                context.stroke().unwrap();
                            }

                        } else {
                            panic!("can not get pangocairo layout");
                        }
                        context.restore().unwrap();
                    }
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use crate::cairo_plotter::rgba_color;

    #[test]
    fn convert_color() {
        assert_eq!("#000000FF", rgba_color((0.0, 0.0, 0.0, 1.0)));
        assert_eq!("#FFFFFFFF", rgba_color((1.0, 1.0, 1.0, 1.0)));
    }
}
