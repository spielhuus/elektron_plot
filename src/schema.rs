use ndarray::{arr1, arr2, Array1, Array2};

use super::border::draw_border;
use super::cairo_plotter::{Circle, Line, LineCap, PlotItem, Text};
use super::theme::{Theme, Themer, ThemerMerge};
use crate::cairo_plotter::{Arc, Polyline, Rectangle};
use crate::text;
use elektron_sexp::{Graph, SchemaElement, TitleBlock, Schema, Shape, Transform};

macro_rules! get_effects {
    ($orig:expr, $theme:expr) => {
        if let Some(effects) = $orig {
            Themer::get(effects, $theme)
        } else {
            $theme.clone()
        }
    };
}

pub struct SchemaPlot<'a, I> {
    iter: I,
    theme: &'a Theme,
    border: bool,
    schema: &'a Schema,
    title_block: &'a Option<TitleBlock>,
    paper_size: (f64, f64),
}

impl<'a, I> Iterator for SchemaPlot<'a, I>
where
    I: Iterator<Item = &'a SchemaElement>,
{
    type Item = Vec<PlotItem>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.border {
            self.border = false;
                return Some(draw_border(self.title_block, self.paper_size, self.theme).unwrap());
        }
        loop {
            match self.iter.next() {
                Some(SchemaElement::Sheet(sheet)) => {
                    let prop = sheet
                        .property
                        .iter()
                        .find(|p| p.key == "Sheet name")
                        .unwrap();
                    let effects = get_effects!(&prop.effects, &self.theme.effects("text").unwrap());
                    let stroke = Themer::get(&sheet.stroke, &self.theme.stroke("symbol").unwrap());
                    return Some(vec![
                        PlotItem::Text(
                            10,
                            Text::new(
                                sheet.at.clone(),
                                0.0,
                                prop.value.clone(),
                                effects.color,
                                effects.font_size.0,
                                effects.font.as_str(),
                                effects.justify,
                                false,
                            ),
                        ),
                        PlotItem::Rectangle(
                            1,
                            Rectangle::new(
                                arr2(&[
                                    [sheet.at[0], sheet.at[1]],
                                    [sheet.at[0] + sheet.size[0], sheet.at[1] + sheet.size[1]],
                                ]),
                                stroke.color,
                                stroke.width,
                                stroke.linetype,
                                Some(sheet.fill),
                            ),
                        ),
                    ]);
                }
                Some(SchemaElement::Wire(wire)) => {
                    let stroke = self.theme.stroke("wire").unwrap();
                    return Some(vec![
                        (PlotItem::Line(
                            10,
                            Line::new(
                                wire.pts.clone(),
                                stroke.width,
                                stroke.linetype.clone(),
                                LineCap::Butt,
                                stroke.color,
                            ),
                        )),
                    ]);
                }
                Some(SchemaElement::Polyline(line)) => {
                    let stroke = Themer::get(&line.stroke, &self.theme.stroke("bus").unwrap());
                    return Some(vec![
                        (PlotItem::Line(
                            10,
                            Line::new(
                                line.pts.clone(),
                                stroke.width,
                                stroke.linetype.clone(),
                                LineCap::Butt,
                                stroke.color,
                            ),
                        )),
                    ]);
                }
                Some(SchemaElement::Bus(bus)) => {
                    let stroke = Themer::get(&bus.stroke, &self.theme.stroke("bus").unwrap());
                    return Some(vec![
                        (PlotItem::Line(
                            10,
                            Line::new(
                                bus.pts.clone(),
                                stroke.width,
                                stroke.linetype.clone(),
                                LineCap::Butt,
                                stroke.color,
                            ),
                        )),
                    ]);
                }
                Some(SchemaElement::BusEntry(bus)) => {
                    let stroke = Themer::get(&bus.stroke, &self.theme.stroke("bus").unwrap());
                    return Some(vec![
                        (PlotItem::Line(
                            10,
                            Line::new(
                                arr2(&[
                                    [bus.at[0], bus.at[1]],
                                    [bus.at[1] + bus.size[0], bus.at[1] + bus.size[1]],
                                ]),
                                stroke.width,
                                stroke.linetype.clone(),
                                LineCap::Butt,
                                stroke.color,
                            ),
                        )),
                    ]);
                }
                Some(SchemaElement::Text(text)) => {
                    let effects = Themer::get(&text.effects, &self.theme.effects("text").unwrap());
                    let pos: Array1<f64> = text.at.clone();
                    let mut angle: f64 = text.angle;
                    if angle >= 180.0 {
                        //dont know why this is possible
                        angle -= 180.0;
                    }
                    return Some(vec![PlotItem::Text(
                        10,
                        Text::new(
                            pos,
                            angle,
                            text.text.clone(),
                            effects.color,
                            effects.font_size.0,
                            effects.font.as_str(),
                            effects.justify,
                            false,
                        ),
                    )]);
                }
                Some(SchemaElement::NoConnect(no_connect)) => {
                    let stroke = self.theme.stroke("no_connect").unwrap();
                    let pos: Array1<f64> = no_connect.at.clone();
                    let lines1 = arr2(&[[-0.8, 0.8], [0.8, -0.8]]) + &pos;
                    let lines2 = arr2(&[[0.8, 0.8], [-0.8, -0.8]]) + &pos;

                    return Some(vec![
                        (PlotItem::Line(
                            10,
                            Line::new(
                                lines1,
                                stroke.width,
                                stroke.linetype.clone(),
                                LineCap::Butt,
                                stroke.color,
                            ),
                        )),
                        PlotItem::Line(
                            10,
                            Line::new(
                                lines2,
                                stroke.width,
                                stroke.linetype,
                                LineCap::Butt,
                                stroke.color,
                            ),
                        ),
                    ]);
                }
                Some(SchemaElement::Junction(junction)) => {
                    let stroke = self.theme.stroke("junction").unwrap();
                    return Some(vec![PlotItem::Circle(
                        99,
                        Circle::new(
                            junction.at.clone(),
                            0.35,
                            stroke.width,
                            stroke.linetype,
                            stroke.color,
                            Option::from(stroke.color),
                        ),
                    )]);
                }
                Some(SchemaElement::Label(label)) => {
                    let effects =
                        Themer::get(&label.effects, &self.theme.effects("label").unwrap());
                    let pos: Array1<f64> = label.at.clone();
                    let mut angle: f64 = label.angle;
                    if angle >= 180.0 {
                        angle -= 180.0;
                    }
                    return Some(vec![PlotItem::Text(
                        10,
                        Text::new(
                            pos,
                            angle,
                            label.text.clone(),
                            effects.color,
                            effects.font_size.0,
                            effects.font.as_str(),
                            effects.justify,
                            false,
                        ),
                    )]);
                }
                Some(SchemaElement::GlobalLabel(label)) => {
                    let effects = self.theme.effects("global_label").unwrap();
                    let pos: Array1<f64> = label.at.clone();
                    let mut angle: f64 = label.angle;
                    if angle > 180.0 {
                        angle -= 180.0;
                    }
                    return Some(vec![PlotItem::Text(
                        10,
                        Text::new(
                            pos,
                            angle,
                            label.text.clone(),
                            effects.color,
                            effects.font_size.0,
                            effects.font.as_str(),
                            effects.justify,
                            true,
                        ),
                    )]);
                }
                Some(SchemaElement::HierarchicalLabel(label)) => {
                    let effects = self.theme.effects("label").unwrap();
                    let pos: Array1<f64> = label.at.clone();
                    let mut angle: f64 = label.angle;
                    if angle >= 180.0 {
                        angle -= 180.0;
                    }
                    return Some(vec![PlotItem::Text(
                        10,
                        Text::new(
                            pos,
                            angle,
                            label.text.clone(),
                            effects.color,
                            effects.font_size.0,
                            effects.font.as_str(),
                            effects.justify,
                            false,
                        ),
                    )]);
                }
                Some(SchemaElement::Symbol(symbol)) => {
                    if symbol.on_schema {
                        let mut items: Vec<PlotItem> = Vec::new();
                        for property in &symbol.property {
                            let mut effects = get_effects!(
                                &property.effects,
                                &self.theme.effects("property").unwrap()
                            );
                            let mut justify: Vec<String> = Vec::new();
                            for j in effects.justify {
                                if property.angle + symbol.angle >= 180.0
                                    && property.angle + symbol.angle < 360.0
                                    && j == "left"
                                {
                                    justify.push(String::from("right"));
                                } else if (property.angle + symbol.angle).abs() >= 180.0
                                    && property.angle + symbol.angle < 360.0
                                    && j == "right"
                                {
                                    justify.push(String::from("left"));
                                } else {
                                    justify.push(j);
                                }
                            }
                            effects.justify = justify;
                            let prop_angle = if (symbol.angle - property.angle).abs() >= 360.0 {
                                (symbol.angle - property.angle).abs() - 360.0
                            } else {
                                (symbol.angle - property.angle).abs()
                            };
                            if !effects.hide {
                                items.push(text!(
                                    property.at.clone(),
                                    prop_angle.abs(),
                                    property.value.clone(),
                                    effects
                                ));
                            }
                        }
                        if let Some(lib) = self.schema.get_library(&symbol.lib_id) {
                            for _unit in &self.schema.get_library(&symbol.lib_id).unwrap().symbols {
                                if _unit.unit == 0 || _unit.unit == symbol.unit {
                                    for graph in &_unit.graph {
                                        match graph {
                                            Graph::Polyline(polyline) => {
                                                let stroke = Themer::get(
                                                    &polyline.stroke,
                                                    &self.theme.stroke("symbol").unwrap(),
                                                );
                                                // let z: usize = if let None = fill_color { 10 } else { 1 };
                                                items.push(PlotItem::Polyline(
                                                    1,
                                                    Polyline::new(
                                                        Shape::transform(symbol, &polyline.pts),
                                                        stroke.color,
                                                        stroke.width,
                                                        stroke.linetype,
                                                        self.theme.color(&polyline.fill_type),
                                                    ),
                                                ));
                                            }
                                            Graph::Rectangle(rectangle) => {
                                                let stroke = Themer::get(
                                                    &rectangle.stroke,
                                                    &self.theme.stroke("symbol").unwrap(),
                                                );
                                                let start = &rectangle.start;
                                                let end = &rectangle.end;
                                                let pts: Array2<f64> =
                                                    arr2(&[[start[0], start[1]], [end[0], end[1]]]);
                                                // let z: usize = if let None = fill_color { 10 } else { 1 };
                                                items.push(PlotItem::Rectangle(
                                                    1,
                                                    Rectangle::new(
                                                        Shape::transform(symbol, &pts),
                                                        stroke.color,
                                                        stroke.width,
                                                        stroke.linetype,
                                                        self.theme.color(&rectangle.fill_type),
                                                    ),
                                                ));
                                            }
                                            Graph::Circle(circle) => {
                                                let stroke = Themer::get(
                                                    &circle.stroke,
                                                    &self.theme.stroke("symbol").unwrap(),
                                                );
                                                // let z: usize = if let None = fill_color { 10 } else { 1 };
                                                items.push(PlotItem::Circle(
                                                    1,
                                                    Circle::new(
                                                        Shape::transform(symbol, &circle.center),
                                                        circle.radius,
                                                        stroke.width,
                                                        stroke.linetype,
                                                        stroke.color,
                                                        self.theme.color(&circle.fill_type),
                                                    ),
                                                ));
                                            }
                                            Graph::Arc(arc) => {
                                                let stroke = Themer::get(
                                                    &arc.stroke,
                                                    &self.theme.stroke("symbol").unwrap(),
                                                );
                                                // let z: usize = if let None = _fill_color { 10 } else { 1 };
                                                items.push(PlotItem::Arc(
                                                    1,
                                                    Arc::new(
                                                        Shape::transform(symbol, &arc.start),
                                                        arc.mid.clone(),
                                                        arc.end.clone(),
                                                        stroke.width,
                                                        stroke.linetype,
                                                        stroke.color,
                                                        self.theme.color(&arc.fill_type),
                                                    ),
                                                ));
                                            }
                                            Graph::Text(text) => {
                                                let effects = Themer::get(
                                                    &text.effects,
                                                    &self.theme.effects("symbol").unwrap(),
                                                );
                                                // let z: usize = if let None = _fill_color { 10 } else { 1 };
                                                items.push(text!(
                                                    Shape::transform(symbol, &text.at),
                                                    text.angle,
                                                    text.text.clone(),
                                                    effects
                                                ));
                                            }
                                        }
                                    }

                                    for pin in &_unit.pin {
                                        /* if graph.has("hide") {
                                            break;
                                        } */
                                        let stroke = self.theme.stroke("pin").unwrap();
                                        let pin_line: Array2<f64> = arr2(&[
                                            [pin.at[0], pin.at[1]],
                                            [
                                                pin.at[0]
                                                    + pin.angle.to_radians().cos() * pin.length,
                                                pin.at[1]
                                                    + pin.angle.to_radians().sin() * pin.length,
                                            ],
                                        ]);
                                        items.push(PlotItem::Line(
                                            10,
                                            Line::new(
                                                Shape::transform(symbol, &pin_line),
                                                stroke.width,
                                                stroke.linetype,
                                                LineCap::Butt,
                                                stroke.color,
                                            ),
                                        ));

                                        if !lib.power && lib.pin_numbers_show {
                                            let npos = if pin.angle == 0.0 || pin.angle == 180.0 {
                                                arr1(&[
                                                    pin.at[0]
                                                        + pin.angle.to_radians().cos() * pin.length
                                                            / 2.0,
                                                    pin.at[1] - 1.0,
                                                ])
                                            } else {
                                                arr1(&[
                                                    pin.at[0] - 1.0,
                                                    pin.at[1]
                                                        + pin.angle.to_radians().sin() * pin.length
                                                            / 2.0,
                                                ])
                                            };

                                            let effects = self.theme.effects("pin_number").unwrap();
                                            items.push(text!(
                                                Shape::transform(symbol, &npos),
                                                0.0,
                                                pin.number.0.clone(),
                                                effects
                                            ));
                                        }
                                        if !lib.power && pin.name.0 != "~" && lib.pin_names_show {
                                            let name_pos = arr1(&[
                                                pin.at[0]
                                                    + pin.angle.to_radians().cos()
                                                        * (pin.length + lib.pin_names_offset * 8.0),
                                                pin.at[1]
                                                    + pin.angle.to_radians().sin()
                                                        * (pin.length + lib.pin_names_offset * 8.0),
                                            ]);
                                            let effects = self.theme.effects("pin_name").unwrap();
                                            items.push(PlotItem::Text(
                                                99,
                                                Text::new(
                                                    Shape::transform(symbol, &name_pos),
                                                    pin.angle,
                                                    pin.name.0.clone(),
                                                    effects.color,
                                                    effects.font_size.0,
                                                    &effects.font,
                                                    vec![String::from("center")],
                                                    false,
                                                ),
                                            ));
                                        }
                                    }
                                }
                            }
                        } else {
                            let pts = arr2(&[[0.0, 0.0], [10.0, 10.0]]);
                            items.push(PlotItem::Rectangle(
                                10,
                                Rectangle::new(
                                    Shape::transform(symbol, &pts),
                                    (1.0, 0.0, 0.0, 1.0),
                                    0.35,
                                    String::from("default"),
                                    None,
                                ),
                            ));
                        }
                        return Some(items);
                    }
                }
                None => {
                    return None;
                }
            }
        }
        /* } else {
        } */

        /* },
        None => {
            return None;
        },
        _ => {} */
        /* }
        } */
    }
}

impl<'a, I> SchemaPlot<'a, I> {
    pub fn new(iter: I, schema: &'a Schema, title_block: &'a Option<TitleBlock>, paper_size: (f64, f64), theme: &'a Theme, border: bool) -> Self {
        Self {
            iter,
            theme,
            border,
            schema,
            title_block,
            paper_size,
        }
    }
}

pub trait PlotIterator<T>: Iterator<Item = T> + Sized {
    fn plot<'a>(self, schema: &'a Schema, title_block: &'a Option<TitleBlock>, paper_size: (f64, f64), theme: &'a Theme, border: bool) -> SchemaPlot<'a, Self> {
        SchemaPlot::new(self, schema, title_block, paper_size, theme, border)
    }
}
impl<T, I: Iterator<Item = T>> PlotIterator<T> for I {}

#[cfg(test)]
mod tests {
    use elektron_sexp::Schema;
    use std::path::Path;

    use crate::plot_schema;

    #[test]
    fn plt_dco() {
        let doc = Schema::load("files/dco.kicad_sch").unwrap();
        plot_schema(&doc, "/tmp/dco.svg", 3.0, false, "kicad_2000").unwrap();
        assert!(Path::new("/tmp/dco.svg").exists());
        assert!(Path::new("/tmp/dco.svg").metadata().unwrap().len() > 0);
    }
    #[test]
    fn plt_dco_mono() {
        let doc = Schema::load("files/dco.kicad_sch").unwrap();
        plot_schema(&doc, "/tmp/dco-mono.svg", 3.0, false, "mono").unwrap();
        assert!(Path::new("/tmp/dco-mono.svg").exists());
        assert!(Path::new("/tmp/dco-mono.svg").metadata().unwrap().len() > 0);
    }
    #[test]
    fn plt_summe() {
        let doc = Schema::load("files/summe.kicad_sch").unwrap();
        plot_schema(&doc, "/tmp/summe.svg", 3.0, true, "kicad_2000").unwrap();
        assert!(Path::new("/tmp/summe.svg").exists());
        assert!(Path::new("/tmp/summe.svg").metadata().unwrap().len() > 0);
    }
    #[test]
    fn plt_summe_mono() {
        let doc = Schema::load("files/summe.kicad_sch").unwrap();
        plot_schema(&doc, "/tmp/summe-mono.svg", 3.0, true, "mono").unwrap();
        assert!(Path::new("/tmp/summe-mono.svg").exists());
        assert!(Path::new("/tmp/summe-mono.svg").metadata().unwrap().len() > 0);
    }
}
