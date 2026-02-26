use std::env;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::ops::Index;
use anyhow::Context;
use anyhow::{Result, Ok};
use chrono::Duration;
use chrono::NaiveTime;
use chrono::Timelike;
use plotters::style::full_palette::BLUE;
use plotters::style::full_palette::BLUE_100;
use plotters::style::full_palette::BLUE_500;
use plotters::style::full_palette::GREEN_100;
use plotters::style::full_palette::GREEN_500;
use plotters::style::full_palette::RED_100;
use plotters::style::full_palette::RED_500;
use regex::Regex;
use serde_json::from_str;
use serde_json::{Value};
use serde::{Deserialize, Serialize};
use plotters::prelude::*;
use chrono::{NaiveDate, NaiveDateTime};
use plotters::data::fitting_range;

const OUT_FILE_NAME: &str = "plot.png";

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
struct BleRSSI {
    time: String,
    device: String,
    rssi: i32,
    address: String,
}

fn plot() -> anyhow::Result<()> {
    // unwrapping here cause if these files don't exist i've died or something
    let file1 = File::open("LOG-LE_1M-0dBm.json").unwrap();
    let file2 = File::open("LOG-LE_1M-4dBm.json").unwrap();
    let file3 = File::open("LOG-kitchen-8dBm.json").unwrap();

    let zero_dbm: Vec<BleRSSI> = serde_json::from_reader(file1).unwrap();
    let four_dbm: Vec<BleRSSI> = serde_json::from_reader(file2).unwrap();
    let eight_dbm: Vec<BleRSSI> = serde_json::from_reader(file3).unwrap();

    // node 1
    let zero_nodeone: Vec<&BleRSSI> = zero_dbm.iter().clone().filter(|a|a.address == "FF:E4:05:1A:8F:FF").collect();
    let four_nodeone: Vec<&BleRSSI> = four_dbm.iter().clone().filter(|a|a.address == "FF:E4:05:1A:8F:FF").collect();
    let eight_nodeone: Vec<&BleRSSI> = eight_dbm.iter().clone().filter(|a|a.address == "FF:E4:05:1A:8F:FF").collect();
    
    // node 2
    let zero_node: Vec<&BleRSSI> = zero_dbm.iter().clone().filter(|a|a.address == "FF:E4:06:1A:8F:FF").collect();
    let four_node: Vec<&BleRSSI> = four_dbm.iter().clone().filter(|a|a.address == "FF:E4:06:1A:8F:FF").collect();
    let eight_node: Vec<&BleRSSI> = eight_dbm.iter().clone().filter(|a|a.address == "FF:E4:06:1A:8F:FF").collect();

    let parse_from_str = NaiveDateTime::parse_from_str;
    let corrected_time: Vec<NaiveTime> = zero_node.clone().iter().map(|a|{
        let dot = a.time.find(".").unwrap();
        let meow = a.time.split_at(dot);
        let hundredths = meow.1[1..3].parse::<u32>().unwrap_or(0) * 10_000_000; // Convert hundredths to nanoseconds
        let base_time = parse_from_str(&format!("2026-01-28 {}", meow.0), "%Y-%m-%d %H:%M:%S").unwrap();
        base_time.time() + chrono::Duration::nanoseconds(hundredths.into())
    }).collect();

    let four_corrected_time: Vec<NaiveTime> = four_node.clone().iter().map(|a|{
        let dot = a.time.find(".").unwrap();
        let meow = a.time.split_at(dot);
        let hundredths = meow.1[1..3].parse::<u32>().unwrap_or(0) * 10_000_000; // Convert hundredths to nanoseconds
        let base_time = parse_from_str(&format!("2026-01-28 {}", meow.0), "%Y-%m-%d %H:%M:%S").unwrap();
        // println!("{base_time:?}");
        base_time.time() + chrono::Duration::nanoseconds(hundredths.into())
    }).collect();

    let eight_corrected_time: Vec<NaiveTime> = eight_node.clone().iter().map(|a|{
        let dot = a.time.find(".").unwrap();
        let meow = a.time.split_at(dot);
        let hundredths = meow.1[1..3].parse::<u32>().unwrap_or(0) * 10_000_000; // Convert hundredths to nanoseconds
        let base_time = parse_from_str(&format!("2026-01-28 {}", meow.0), "%Y-%m-%d %H:%M:%S").unwrap();
        base_time.time() + chrono::Duration::nanoseconds(hundredths.into())
    }).collect();

    let time_start = corrected_time.first().unwrap().num_seconds_from_midnight();
    let time_end = eight_corrected_time.last().unwrap().num_seconds_from_midnight();

    let root_area = BitMapBackend::new(OUT_FILE_NAME, (2048, 1536)).into_drawing_area();

    root_area.fill(&WHITE)?;

    let root_area = root_area.titled("RSSI plot", ("sans-serif", 60))?;

    let (upper, lower) = root_area.split_vertically(604);

    let mut cc = ChartBuilder::on(&upper)
        .margin(2)
        .set_all_label_area_size(50)
        .build_cartesian_2d(time_start as f32..time_end as f32 - 0f32, -120f32..0f32)?;

    cc.configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .disable_mesh()
        .x_label_formatter(&|v| {
            let seconds = *v;
            let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap() + Duration::seconds(seconds as i64);
            format!("{}", time.format("%H:%M:%S"))
        })
        .draw()?;

    // LIVING ROOM NODE
    cc.draw_series(LineSeries::new(zero_node.iter().zip(corrected_time.iter()).map(|(node, time)| {
            let seconds = time.num_seconds_from_midnight();
            (seconds as f32, node.rssi as f32)
        }), &BLUE_500))?
        .label("node living room: zero dBm TX power")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE_500));

    cc.draw_series(LineSeries::new(four_node.iter().zip(four_corrected_time.iter()).map(|(node, time)| {
            let seconds = time.num_seconds_from_midnight();
            (seconds as f32, node.rssi as f32)
        }), &RED_500))?
        .label("node living room: 4 dBm TX power")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED_500));

    cc.draw_series(LineSeries::new(eight_node.iter().zip(eight_corrected_time.iter()).map(|(node, time)| {
            let seconds = time.num_seconds_from_midnight();
            (seconds as f32, node.rssi as f32)
        }), &GREEN_500))?
        .label("node living room: 8 dBm TX power")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN_500));

    // KITCHEN NODE
    cc.draw_series(LineSeries::new(zero_nodeone.iter().zip(corrected_time.iter()).map(|(node, time)| {
            let seconds = time.num_seconds_from_midnight();
            (seconds as f32, node.rssi as f32)
        }), &BLUE_100))?
        .label("node kitchen")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE_100));

    cc.draw_series(LineSeries::new(four_nodeone.iter().zip(four_corrected_time.iter()).map(|(node, time)| {
            let seconds = time.num_seconds_from_midnight();
            (seconds as f32, node.rssi as f32)
        }), &RED_100))?
        .label("node kitchen")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED_100));

    cc.draw_series(LineSeries::new(eight_nodeone.iter().zip(eight_corrected_time.iter()).map(|(node, time)| {
            let seconds = time.num_seconds_from_midnight();
            (seconds as f32, node.rssi as f32)
        }), &GREEN_100))?
        .label("node kitchen")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN_100));

    cc.configure_series_labels().border_style(BLACK).label_font(("Calibri", 20)).draw()?;

    let mut cc = ChartBuilder::on(&lower)
        .margin(2)
        .set_all_label_area_size(50)
        .build_cartesian_2d( 0f32..eight_node.len() as f32, -120f32..0f32)?;

    cc.configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .disable_mesh()
        .draw()?;

    // LIVING ROOM NODE
    cc.draw_series(LineSeries::new(zero_node.iter().enumerate().map(|a| (a.0 as f32, a.1.rssi as f32)), &BLUE_500))?
        .label("node living room: zero dBm TX power")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE_500));

    cc.draw_series(LineSeries::new(four_node.iter().enumerate().map(|a| (a.0 as f32, a.1.rssi as f32)), &RED_500))?
        .label("node living room: 4 dBm TX power")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED_500));

    cc.draw_series(LineSeries::new(eight_node.iter().enumerate().map(|a| (a.0 as f32, a.1.rssi as f32)), &GREEN_500))?
        .label("node living room: 8 dBm TX power")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN_500));


    cc.configure_series_labels().border_style(BLACK).label_font(("Calibri", 20)).draw()?;

    root_area.present().expect("Unable to write result to file");
    println!("Result has been saved to {}", OUT_FILE_NAME);
    Ok(())
}

fn save_file(json_content: &String, filename: &String) -> Result<()> {
    let mut f: File = File::create(filename)?;
    f.write_all(json_content.as_bytes())?;

    Ok(())
}

fn main() -> Result<()> {
    let time = Regex::new(r"[0-9]{2}:[0-9]{2}:[0-9]{2}(\.[0-9]{1,3})").unwrap();
    let name = Regex::new(r"NAME:\s*(.*)").unwrap();
    let rssi = Regex::new(r"RSSI:\s[-+]?\d+").unwrap();
    let mac_addr = Regex::new(r"Address:\s([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})").unwrap();

    let args: Vec<String> = env::args().collect();

    if args[1] == "plot" {
        plot()?;
    } else {

    // next will be 4dBm - writing this at 00:28 am
    let contents = File::open(&args[1]).expect("couldn't open file");
    let target_address_one: &str = &args[2];  // mac address like "FF:E4:06:1A:8F:FF" or "FF:E4:05:1A:8F:FF"
    let target_address_two: &str = &args[3];
    let reader = BufReader::new(contents);

    let mut data = vec![BleRSSI { ..Default::default()}; 0];

    for cursor_result in reader.lines() {
        // let _line_length = reader.read_line(&mut line).expect("couldn't read line!");
        let line = cursor_result.expect("couldn't read line");
        let line = line.trim();

        if time.is_match(&line) {
            data.push(BleRSSI { time: line.to_string(), ..Default::default() });
        } else if name.is_match(&line) {
            data.last_mut().map(|a: &mut BleRSSI| a.device = line[6..].into());
        }  else if rssi.is_match(&line) {
            data.last_mut().map(|a: &mut BleRSSI| a.rssi = line[6..].parse::<i32>().expect("expected an integer in this field"));
        }  else if mac_addr.is_match(&line) {
            data.last_mut().map(|a: &mut BleRSSI| a.address = line[9..].into());
        };
    }
    // println!("output: {:?}", data);

    let filtered_list: Vec<&BleRSSI> = data.iter().filter(|a| a.address == target_address_one || a.address == target_address_two).collect();
    let a = serde_json::to_string_pretty(&filtered_list).context("cannot convert an empty string to json")?;
    save_file(&a, &args[2])?;

    // println!("{}", a);
    }

    Ok(())
}
