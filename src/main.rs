use plotters::prelude::*;

//----------основные константы----------
const СС: f64 = 2.99792458e8; //скорость света
const MUZ: f64 = 4.0 * std::f64::consts::PI * 1e-7; //магнитная проницаемость
const EPSZ: f64 = 1.0 / (СС * СС * MUZ); //диэлектрическая проницаемость
const FREQ: f64 = 1.0e9; //частота источника возбуждения
const LAMBDA: f64 = СС / FREQ; //длина волны источника возбуждения
const OMEGA: f64 = 2.0 * std::f64::consts::PI * FREQ; 

//----------функция для рисования кадров----------
fn draw_frame(
    path: &str,
    x: &[f64],
    ez: &[f64],
    hy: &[f64],
    t_ns: f64,
    scfact: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let (top, bottom) = root.split_vertically(300);

    let mut chart = ChartBuilder::on(&top)
        .caption(format!("t = {:.0} ns", t_ns), ("sans-serif", 24))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0.0f64..3.0, -1.0f64..1.0)?;
    chart.configure_mesh().draw()?;
    chart.draw_series(LineSeries::new(
        x.iter().zip(ez.iter()).map(|(&xi, &ei)| (xi, ei / scfact)),
        &RED,
    ))?;

    let mut chart = ChartBuilder::on(&bottom)
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0.0f64..3.0, -3.0e-3f64..3.0e-3)?;
    chart.configure_mesh().draw()?;
    chart.draw_series(LineSeries::new(
        x.iter().zip(hy.iter()).map(|(&xi, &hi)| (xi, hi)),
        &BLUE,
    ))?;

    root.present()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = "frames";
    std::fs::create_dir_all(dir)?;
    //----------параметры сетки----------
    let ie: usize = 200; //количество ячеек по оси x
    let ih: usize = ie - 1; //количество ячеек по оси y

    let dx: f64 = LAMBDA / 20.0; //шаг сетки по оси x
    let dt: f64 = dx / СС; //шаг по времени
    let omegadt: f64 = OMEGA * dt; 

    let nmax: usize = (12.0e-9 / dt).round() as usize; //количество временных шагов

    //для сетки по оси x и y создаем массивы координат
    let xarr: Vec<f64> = (1..ie).map(|i| i as f64 * dx).collect(); //массив координат по оси x
    //-----------материалы----------
    let eps = 1.0;
    let sig = 5.0e-3; //проводимость материала

    let scfact = (dt / MUZ) / dx; //коэффициент для обновления электрического поля
    
    let ca = (1.0-(dt*sig)/(2.0*EPSZ*eps))/(1.0+(dt*sig)/(2.0*EPSZ*eps));
    let cb = scfact * (dt / (EPSZ * eps * dx)) / (1.0 + (dt * sig) / (2.0 * EPSZ * eps));

    //----------полевые векторы----------
    let mut ez = vec![0.0f64;ie]; //вектор электрического поля
    let mut hy = vec![0.0f64;ih]; //вектор магнитного поля

    //---------временный цикл----------
    for n in 1..=nmax {
        ez[0] = scfact * (omegadt * n as f64).sin();

        let rbc = ez[ih - 1];
        for i in 1..ih {
            ez[i] = ca * ez[i] + cb * (hy[i] - hy[i - 1]);
        }
        ez[ie - 1] = rbc;

        for i in 0..ih {
            hy[i] = hy[i] + ez[i + 1] - ez[i];
        }

        if n % 10 == 0 {
            let t_ns = (n as f64 * dt) * 1.0e9;
            let path = format!("{}/frame_{:04}.png", dir, n / 10);
            draw_frame(&path, &xarr, &ez, &hy, t_ns, scfact)?;
        }
    }
    Ok(())
}
