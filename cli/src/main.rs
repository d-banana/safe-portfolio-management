use plotters::prelude::*;
use simulation::generate_price_graph;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = generate_price_graph();
    let y_min = data.iter().min_by(|x, y| x.1.total_cmp(&y.1)).unwrap().1 * 0.98;
    let y_max = data.iter().max_by(|x, y| x.1.total_cmp(&y.1)).unwrap().1 * 1.02;
    let root = BitMapBackend::new("graph/0.png", (1920, 1080)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Price chart", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(60)
        .y_label_area_size(60)
        .build_cartesian_2d(
            data.first().unwrap().0..data.last().unwrap().0,
            y_min..y_max,
        )?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(data.into_iter(), &RED))?
        .label("price")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}
