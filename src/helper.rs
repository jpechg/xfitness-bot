use std::collections::HashMap;
use std::io::Cursor;
use teloxide::types::InputFile;
use chrono::{Timelike, NaiveDate, NaiveDateTime, Locale, Local};
use serde::Deserialize;
use serde_json::Result;
use plotters::prelude::*;
use image::*;


#[derive(Deserialize, Debug)]
struct AforoJson {
    Hora : u64,
    OcupacionActual: u64,
    OcupacionPrevista: u64,
}

pub struct Aforo {
    pub time: NaiveDateTime,
    pub realpeople: u64,
    pub expectedpeople: u64,
}

#[derive(Deserialize, Debug)]
struct ResponseJson {
    codigoRespuesta: u16,
    datos: DatosJson,
}

#[derive(Deserialize, Debug)]
struct DatosJson {
    Aforos: Vec<AforoJson>,
}

pub async fn deserialize(day: NaiveDate) -> std::result::Result<Vec<Aforo>, reqwest::Error> {

    let day = day.format("%Y-%m-%d").to_string();
    //println!("parsed day: {}", day);

    let url = format!("{}{}", "https://basic.deporweb.net/api/aforo/v1/previsto/MjN8MQ==/", day);
    //println!("{}", url);
    //println!("completed get request");

    let response = reqwest::get(url.clone()).await?;
    let json = response.text().await?.replace("null","0");

    //println!("{}, {}", url, json);
    //test data
    //let json = r#"
    //{"codigoRespuesta":200,"datos":{"Aforos":[{"Hora":6,"OcupacionActual":2,"OcupacionPrevista":3},{"Hora":7,"OcupacionActual":14,"OcupacionPrevista":18},{"Hora":8,"OcupacionActual":15,"OcupacionPrevista":16},{"Hora":9,"OcupacionActual":9,"OcupacionPrevista":9},{"Hora":10,"OcupacionActual":11,"OcupacionPrevista":12},{"Hora":11,"OcupacionActual":18,"OcupacionPrevista":13},{"Hora":12,"OcupacionActual":18,"OcupacionPrevista":12},{"Hora":13,"OcupacionActual":18,"OcupacionPrevista":10},{"Hora":14,"OcupacionActual":21,"OcupacionPrevista":12},{"Hora":15,"OcupacionActual":20,"OcupacionPrevista":16},{"Hora":16,"OcupacionActual":19,"OcupacionPrevista":18},{"Hora":17,"OcupacionActual":16,"OcupacionPrevista":22},{"Hora":18,"OcupacionActual":33,"OcupacionPrevista":32},{"Hora":19,"OcupacionActual":39,"OcupacionPrevista":43},{"Hora":20,"OcupacionActual":40,"OcupacionPrevista":45},{"Hora":21,"OcupacionActual":24,"OcupacionPrevista":32},{"Hora":22,"OcupacionActual":12,"OcupacionPrevista":20},{"Hora":23,"OcupacionActual":3,"OcupacionPrevista":5}]},"error":false,"mensaje":"Datos obtenidos correctamente"}
    //"#;

    let mut aforo: Vec<Aforo> = Vec::new();
    let aforo_json: ResponseJson = match serde_json::from_str(&json) {
        Ok(data) => data,
        Err(e) => {
            println!("Failed to deserialize JSON: {}", e);
            return Ok(Vec::new());
        }
    };

    let lenaforos = aforo_json.datos.Aforos.len();
    let mut fecha: String;
    //debido a un error en el api, el gym abre a las 7, no a las 6, asi q la primera hora no vale para nada
    for n in 1..lenaforos {
        fecha = format!("{} {}:0:0", day, aforo_json.datos.Aforos[n].Hora.to_string());
        aforo.push(
            Aforo {
                time: NaiveDateTime::parse_from_str(&fecha, "%Y-%m-%d %H:%M:%S").unwrap(),
                realpeople: aforo_json.datos.Aforos[n].OcupacionActual,
                expectedpeople: aforo_json.datos.Aforos[n].OcupacionPrevista,
            })
    }

    Ok(aforo)
}

pub async fn average_expected_users(aforos: &Vec<Aforo>) -> u64 {

    let mut total: u64 = 0;
    for n in 0..aforos.len() {
        total = total + aforos[n].expectedpeople;
    }
    let average = total/(aforos.len() as u64);

    average
}

pub async fn average_real_users(aforos: &Vec<Aforo>) -> u64 {

    let mut total: u64 = 0;
    let mut effective_hours: u64 = 0;
    
    for n in 0..aforos.len() {
        if aforos[n].realpeople != 0 {
            total = total + aforos[n].realpeople;
            effective_hours = effective_hours + 1;
        }
    }

    if effective_hours == 0 {
        0
    }
    else {
        total/effective_hours
    }
}
pub async fn users_at_hour(aforos: &Vec<Aforo>, hora: u8) -> u64 {

    let index: usize = (6 + hora) as usize;
    // the first hour is 7 AM, as we already discard the first hour in which the gym is never open
    
    if aforos[index].realpeople != 0 {
        
        aforos[index].realpeople
    
    }
    
    else {
        
        aforos[index].expectedpeople
    
    }    
}

pub async fn current_users(aforos: &Vec<Aforo>, time: NaiveDateTime) -> u64 {

    let hora: u64 = time.hour() as u64;
    println!("{}", hora);
    // the first hour is 7 AM, as we already discard the first hour in which the gym is never open
    // Prevent an exception if the bot is used while the gym is not open
    if !(7..23).contains(&hora) {
        
        return 0
    
    }
    
    else {
        
        let index: usize = (hora - 7) as usize;
        let current_users = aforos[index].realpeople;

        current_users
    
    }
}

pub async fn histogram_days(days: u32) -> InputFile {
    // Get the current date
    let now = Local::now().date_naive();

    // Create a vector to hold the data for the histogram
    let mut data = Vec::new();

    // Create a 
    // Get the data for the next `n` days
    for day in 0..days {
        let date = now + chrono::Duration::days(day as i64);
        let aforos = deserialize(date).await.unwrap();
        let average = average_expected_users(&aforos).await as i32;
        data.push(average);
    }

    
    let mut img: Vec<u8> = vec![0; 1200 * 800 * 3];
    {
        let root_area = BitMapBackend::with_buffer(&mut img, (1200, 800))
        .into_drawing_area();
        root_area.fill(&RGBColor(255,255,255)).unwrap();

        let mut ctx = ChartBuilder::on(&root_area)
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .caption("Aforo esperado por días", ("sans-serif", 60))
            .build_cartesian_2d((0..(data.len()-1)).into_segmented(), 0..40)
            .unwrap();

        ctx.configure_mesh()
        .x_label_formatter(&|v| {
            let day = match &v {
                SegmentValue::Exact(value) => now + chrono::Duration::days(value.clone() as i64),
                // we only care about the exact value, both are placeholder values
                SegmentValue::Last => now,
                SegmentValue::CenterOf(value) => now + chrono::Duration::days(value.clone() as i64),
                };
            format!("{}", day.format_localized("%a %d",Locale::es_ES)) // %a is the format for abbreviated weekday name
        })
        .x_labels(10)
        .y_labels(10)
        .x_label_style(("sans-serif", 40).into_font()) // Adjust the font size here
        .y_label_style(("sans-serif", 40).into_font()) // Adjust the font size here
        .draw().unwrap();

        ctx.draw_series((0..).zip(data.iter()).map(|(x, y)| {
        let x0 = SegmentValue::Exact(x);
        let x1 = SegmentValue::Exact(x + 1);
        let mut bar = Rectangle::new([(x0, 0), (x1, *y)], RGBColor(0,139,139).filled());
        bar.set_margin(0, 0, 5, 5);
        bar
        }))
        .unwrap();

         ctx.configure_series_labels()
        .label_font(("sans-serif", 50).into_font()) // Adjust the font size here
        .draw()
        .unwrap();

        //for (x, y) in (0..).zip(data.iter()) {
        //    let text = Text::new(format!("{}", y),(x as i32, *y as i32),("sans-serif", 20.0).into_font(),);
        //    root_area.draw(&text).unwrap();
        //}
    }
    let img = DynamicImage::ImageRgb8(ImageBuffer::from_raw(1200, 800, img).unwrap());
        let mut png: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    img.write_to(&mut png, image::ImageOutputFormat::Png)
        .expect("Failed to convert image to png");

    let photo = teloxide::types::InputFile::memory(png.into_inner());
    
    photo
}

pub async fn histogram_today() -> InputFile {
    // Get the current date
    let now = Local::now().date_naive();

    // Create a vector to hold the data for the histogram
    let expectedpeople: Vec<i32> = deserialize(now).await.unwrap()
        .into_iter()
        .map(|Aforo { expectedpeople, .. }| expectedpeople as i32)
        .collect();

    let realpeople : Vec<i32> = deserialize(now).await.unwrap()
        .into_iter()
        .map(|Aforo { realpeople, .. }| realpeople as i32)
        .collect();
    
    //test data    
    //for n in 0..expectedpeople.len() {
    //    println!("hour:{}, expected: {}, real: {}", n+7 , expectedpeople[n], realpeople[n])
    //}
        
    let mut img: Vec<u8> = vec![0; 1200 * 800 * 3];
    {
        let root_area = BitMapBackend::with_buffer(&mut img, (1200, 800))
        .into_drawing_area();
        root_area.fill(&RGBColor(255,255,255)).unwrap();

        let mut ctx = ChartBuilder::on(&root_area)
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .caption("Aforo esperado por horas", ("sans-serif", 60))
            .build_cartesian_2d((0..(2*expectedpeople.len()-1)).into_segmented(), 0..30)
            .unwrap();

        ctx.configure_mesh()
            .x_label_formatter(&|x| {
                match x {
                    SegmentValue::Exact(val) => format!("{}", ((*val as i32) / 2)+7),
                    SegmentValue::CenterOf(val) => format!("{}", ((*val as i32) / 2)+7),
                    SegmentValue::Last => format!("")
                }
            })
            .y_labels(10)
            .x_label_style(("sans-serif", 40).into_font()) // Adjust the font size here
            .y_label_style(("sans-serif", 40).into_font()) // Adjust the font size here
            .draw().unwrap();

        ctx.draw_series((0..).zip(expectedpeople.iter()).map(|(x, y)| {
            let x0 = SegmentValue::Exact(2 * x);
            let x1 = SegmentValue::Exact((2 * x) + 1);
            let mut bar = Rectangle::new([(x0, 0), (x1, *y)], RGBColor(33, 158, 188).filled());
            bar.set_margin(0, 0, 5, 0);
            bar
            }))
            .unwrap()       
            .label("Aforo real")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 30, y)], &RGBColor(33, 158, 188)));
            
        ctx.draw_series((0..).zip(realpeople.iter()).map(|(x, y)| {
            let x0 = SegmentValue::Exact((2 * x) + 1);
            let x1 = SegmentValue::Exact((2 * x) + 2);
            let mut bar = Rectangle::new([(x0, 0), (x1, *y)], RGBColor(2, 48, 71).filled());
            bar.set_margin(0, 0, 0, 5);
            bar
            }))
            .unwrap()
            .label("Aforo real")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 30, y)], &RGBColor(2, 48, 71)));
        
        ctx.configure_series_labels()
            .label_font(("sans-serif", 30).into_font()) // Adjust the font size here
            .border_style(&BLACK)
            .background_style(&WHITE.mix(0.8))
            .draw()
            .unwrap();
    }
    let img = DynamicImage::ImageRgb8(ImageBuffer::from_raw(1200, 800, img).unwrap());
        let mut png: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    img.write_to(&mut png, image::ImageOutputFormat::Png)
        .expect("Failed to convert image to png");

    let photo = teloxide::types::InputFile::memory(png.into_inner());
    
    photo
}

pub async fn emojify_number (num: u64) -> String {

    let mut emoji_map = HashMap::new();
    emoji_map.insert(0, "0️⃣");
    emoji_map.insert(1, "1️⃣");
    emoji_map.insert(2, "2️⃣");
    emoji_map.insert(3, "3️⃣");
    emoji_map.insert(4, "4️⃣");
    emoji_map.insert(5, "5️⃣");
    emoji_map.insert(6, "6️⃣");
    emoji_map.insert(7, "7️⃣");
    emoji_map.insert(8, "8️⃣");
    emoji_map.insert(9, "9️⃣");

    let number_string = num.to_string();
    number_string.chars().map(|c| *emoji_map.get(&(c as u8 - '0' as u8)).unwrap()).collect()

}
