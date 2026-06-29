use plotters::prelude::*;
use plotters::style::RGBColor;

const PALETTE: [RGBColor; 8] = [
    RGBColor(74, 144, 217),
    RGBColor(217, 119, 74),
    RGBColor(74, 180, 120),
    RGBColor(200, 74, 200),
    RGBColor(217, 190, 74),
    RGBColor(74, 190, 200),
    RGBColor(200, 120, 74),
    RGBColor(120, 200, 74),
];

pub struct ChartData {
    pub labels: Vec<String>,
    pub values: Vec<f64>,
}

pub struct MultiSeriesData {
    pub labels: Vec<String>,
    pub series: Vec<(String, Vec<f64>)>,
}

pub struct PieData {
    pub labels: Vec<String>,
    pub values: Vec<f64>,
    pub colors: Vec<RGBColor>,
}

pub struct GaugeData {
    pub value: f64,
    pub max: f64,
    pub label: String,
}

pub struct ConfidenceData {
    pub labels: Vec<String>,
    pub predicted: Vec<f64>,
    pub lower: Vec<f64>,
    pub upper: Vec<f64>,
}

pub fn render_line_chart(data: &ChartData, width: u32, height: u32) -> String {
    let mut buffer = String::new();
    {
        let root = SVGBackend::new(&mut buffer, (width, height)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.margin(15, 15, 15, 15);
        let max_val = data.values.iter().cloned().fold(0.0f64, f64::max);
        let mut chart = ChartBuilder::on(&root)
            .build_cartesian_2d(0..data.values.len(), 0.0..max_val * 1.1)
            .unwrap();
        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .draw()
            .unwrap();
        chart
            .draw_series(LineSeries::new(
                data.values.iter().enumerate().map(|(i, &v)| (i, v)),
                &PALETTE[0],
            ))
            .unwrap()
            .label(format!("Series"))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &PALETTE[0]));
        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()
            .unwrap();
        root.present().unwrap();
    }
    buffer
}

pub fn render_bar_chart(data: &ChartData, width: u32, height: u32) -> String {
    let mut buffer = String::new();
    {
        let root = SVGBackend::new(&mut buffer, (width, height)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.margin(15, 15, 15, 15);
        let max_val = data.values.iter().cloned().fold(0.0f64, f64::max);
        let mut chart = ChartBuilder::on(&root)
            .build_cartesian_2d(0..data.values.len(), 0.0..max_val * 1.15)
            .unwrap();
        chart.configure_mesh().draw().unwrap();
        chart
            .draw_series(
                data.values
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| Rectangle::new([(i, 0.0), (i + 1, v)], PALETTE[i % 8].filled())),
            )
            .unwrap();
        root.present().unwrap();
    }
    buffer
}

pub fn render_stacked_bar_chart(data: &MultiSeriesData, width: u32, height: u32) -> String {
    let mut buffer = String::new();
    {
        let root = SVGBackend::new(&mut buffer, (width, height)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.margin(15, 15, 15, 15);
        let mut stacked: Vec<f64> = vec![0.0; data.labels.len()];
        for s in &data.series {
            for (i, &v) in s.1.iter().enumerate() {
                stacked[i] += v;
            }
        }
        let max_val = stacked.iter().cloned().fold(0.0f64, f64::max);
        let mut chart = ChartBuilder::on(&root)
            .build_cartesian_2d(0..data.labels.len(), 0.0..max_val * 1.15)
            .unwrap();
        chart.configure_mesh().draw().unwrap();
        let mut offsets = vec![0.0f64; data.labels.len()];
        for (si, s) in data.series.iter().enumerate() {
            let color = PALETTE[si % 8];
            chart
                .draw_series(s.1.iter().enumerate().map(|(i, &v)| {
                    let y0 = offsets[i];
                    let y1 = y0 + v;
                    offsets[i] = y1;
                    Rectangle::new([(i, y0), (i + 1, y1)], color.filled())
                }))
                .unwrap();
        }
        root.present().unwrap();
    }
    buffer
}

pub fn render_pie_chart(data: &PieData, width: u32, height: u32) -> String {
    let mut buffer = String::new();
    {
        let root = SVGBackend::new(&mut buffer, (width, height)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let total: f64 = data.values.iter().sum();
        let center = (width as i32 / 2, height as i32 / 2);
        let radius = (width.min(height) as f64 / 2.0 * 0.7) as i32;
        let mut start_pct = 0.0f64;
        for (i, &v) in data.values.iter().enumerate() {
            let sweep_pct = v / total;
            let color = data.colors.get(i).copied().unwrap_or(PALETTE[i % 8]);
            let points: Vec<(i32, i32)> = (0..=50)
                .map(|step| {
                    let angle = (start_pct + sweep_pct * (step as f64 / 50.0)) * std::f64::consts::PI * 2.0
                        - std::f64::consts::PI;
                    let x = center.0 + (radius as f64 * angle.cos()) as i32;
                    let y = center.1 + (radius as f64 * angle.sin()) as i32;
                    (x, y)
                })
                .collect();
            root.draw(&Polygon::new(points, color.filled())).unwrap();
            start_pct += sweep_pct;
        }
        root.present().unwrap();
    }
    buffer
}

pub fn render_gauge(data: &GaugeData, width: u32, height: u32) -> String {
    let mut buffer = String::new();
    {
        let root = SVGBackend::new(&mut buffer, (width, height)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let center = (width as i32 / 2, (height as f64 * 0.75) as i32);
        let radius = (width.min(height) as f64 / 2.0 * 0.7) as i32;
        let ratio = (data.value / data.max).min(1.0);
        let arc_color = if ratio > 0.7 {
            PALETTE[2]
        } else if ratio > 0.4 {
            PALETTE[4]
        } else {
            RGBColor(217, 74, 74)
        };
        let bg_points: Vec<(i32, i32)> = (0..=50)
            .map(|step| {
                let angle = (step as f64 / 50.0) * std::f64::consts::PI - std::f64::consts::PI;
                let x = center.0 + (radius as f64 * angle.cos()) as i32;
                let y = center.1 + (radius as f64 * angle.sin()) as i32;
                (x, y)
            })
            .collect();
        root.draw(&Polygon::new(bg_points, RGBColor(230, 230, 230).filled()))
            .unwrap();
        let fg_points: Vec<(i32, i32)> = (0..=50)
            .map(|step| {
                let angle = (step as f64 / 50.0) * std::f64::consts::PI - std::f64::consts::PI;
                let x = center.0 + (radius as f64 * angle.cos()) as i32;
                let y = center.1 + (radius as f64 * angle.sin()) as i32;
                (x, y)
            })
            .take((ratio * 50.0) as usize + 1)
            .collect();
        if fg_points.len() >= 2 {
            root.draw(&Polygon::new(fg_points, arc_color.filled())).unwrap();
        }
        root.present().unwrap();
    }
    buffer
}

pub fn render_confidence_chart(data: &ConfidenceData, width: u32, height: u32) -> String {
    let mut buffer = String::new();
    {
        let root = SVGBackend::new(&mut buffer, (width, height)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.margin(15, 15, 15, 15);
        let all_vals: Vec<f64> = data
            .upper
            .iter()
            .chain(data.lower.iter())
            .copied()
            .collect();
        let max_val = all_vals.iter().cloned().fold(0.0f64, f64::max);
        let min_val = all_vals.iter().cloned().fold(f64::INFINITY, f64::min);
        let lo = if min_val == f64::INFINITY { 0.0 } else { min_val * 0.9 };
        let hi = if max_val == 0.0 { 100.0 } else { max_val * 1.1 };
        let mut chart = ChartBuilder::on(&root)
            .build_cartesian_2d(0..data.labels.len(), lo..hi)
            .unwrap();
        chart.configure_mesh().draw().unwrap();
        for i in 0..data.lower.len() {
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(i, data.lower[i]), (i + 1, data.upper[i])],
                    PALETTE[0].mix(0.2).filled(),
                )))
                .unwrap();
        }
        chart
            .draw_series(LineSeries::new(
                data.predicted
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| (i, v)),
                &PALETTE[0],
            ))
            .unwrap();
        root.present().unwrap();
    }
    buffer
}
