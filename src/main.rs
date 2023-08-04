// use std::os::unix::process;

use sysinfo::{ProcessExt, DiskExt, System, SystemExt};
use rusoto_core::{Region, RusotoError, request};
use rusoto_cloudwatch::{CloudWatch, CloudWatchClient, Dimension, MetricDatum, PutMetricDataInput, PutMetricDataError};
use tokio::runtime::Runtime;
use std::time::Instant;
use tokio::time::{timeout, Duration, sleep};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use serde_yaml::Error;
use reqwest::Client;

#[derive(Deserialize, Debug)]
struct Config {
    region: String,
    process_checks: Vec<String>,
}

fn read_config(filename: &str) -> Result<Config, Error> {
    let mut file = File::open(filename).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let config: Config = serde_yaml::from_str(&contents)?;

    Ok(config)
}

// #[derive(Debug)]
// struct Disk {
//     name: String,
//     mount_point: String,
//     // fs: Vec<char>,
//     // disk_type: String,
//     // removable: bool,
//     used: u64,
//     total: u64,
// }

// fn parse_disk_data(sys: &System) -> Vec<Disk> {
//     sys.disks().iter().map(|disk_info| {
//         let name = disk_info.name().to_str().unwrap().to_string();
//         let mount_point = disk_info.mount_point().to_string_lossy().into_owned();
//         // let mount_point = disk_info.mount_point().to_str().unwrap().to_string();

//         // These values are placeholders as sysinfo doesn't provide them
//         // let fs = vec!['a', 'p', 'f', 's'];
//         // let disk_type = "SSD".to_string();
//         // let removable = false;

//         let used = disk_info.total_space() - disk_info.available_space();
//         let total = disk_info.total_space();

//         Disk {
//             name,
//             mount_point,
//             // fs,
//             // disk_type,
//             // removable,
//             used,
//             total,
//         }
//     }).collect()
// }


// fn get_disk_data() {
//     let mut sys = System::new_all();

//     // We refresh all information of our system
//     sys.refresh_all();

//     for disk in sys.disks() {
//         println!("{:?}", disk);
//     }
// }

// async fn send_metric_to_cloudwatch(proc_status: &str, namespace: &str, metric_name: &str) -> Result<(), RusotoError<PutMetricDataError>> {
//     let client = CloudWatchClient::new(Region::EuWest2); // choose your AWS region

//     let metric_value = if proc_status == "Run" { 0.0 } else { 1.0 };

//     let dimension = Dimension {
//         name: "ProcessStatus".to_string(),
//         value: proc_status.to_string(),
//     };

//     let datum = MetricDatum {
//         dimensions: Some(vec![dimension]),
//         metric_name: metric_name.to_string(),
//         value: Some(metric_value),
//         ..Default::default()
//     };

//     let put_metric_data_req = PutMetricDataInput {
//         namespace: namespace.to_string(),
//         metric_data: vec![datum],
//     };

//     println!("Sending the following data to AWS CloudWatch: {:?}", put_metric_data_req);

//     match client.put_metric_data(put_metric_data_req).await {
//         Ok(_) => {
//             println!("Metric sent to CloudWatch successfully.");
//             return Ok(());
//         },
//         Err(e) => eprintln!("Failed to send metric to CloudWatch: {}", e),
//     }

//     Ok(())
// }


// async fn send_metrics(filter_string: &str) {
//     let sys = System::new_all();

//     let start = Instant::now();
//     for (_pid, proc_) in sys.processes().iter().filter(|(_, proc_)| proc_.name().to_lowercase().contains(filter_string)) {
//         let proc_status = format!("{:?}", proc_.status());

//         let mut retries = 0;
//         loop {
//             // Check if the total time exceeds 30 minutes, if yes break the loop
//             if start.elapsed().as_secs() > 30 * 60 {
//                 break;
//             }

//             // Add timeout
//             let send_result = timeout(Duration::from_secs(10), send_metric_to_cloudwatch(&proc_status, "Rust", filter_string)).await;

//             match send_result {
//                 Ok(result) => match result {
//                     Ok(_) => break, // success, break the loop
//                     Err(e) => {
//                         eprintln!("Error when sending metric to CloudWatch: {:?}", e);
//                         retries += 1;
//                         if retries >= 3 { // if number of retries exceed 3, break the loop
//                             eprintln!("Failed to send metric after {} retries", retries);
//                             break;
//                         }
//                     },
//                 },
//                 Err(e) => {
//                     eprintln!("Timeout when sending metric to CloudWatch: {:?}", e);
//                     retries += 1;
//                     if retries >= 3 { // if number of retries exceed 3, break the loop
//                         eprintln!("Failed to send metric after {} retries", retries);
//                         break;
//                     }
//                 },
//             }

//             // sleep for 1 second
//             sleep(Duration::from_secs(180)).await;
//         }
//     }
// }

async fn send_metrics_with_retries(filter_string: &str, region: &str) -> Result<(), Box<dyn std::error::Error>> {
    let sys = System::new_all();

    let start = Instant::now();

    let instance_id = reqwest::get("http://169.254.169.254/latest/meta-data/instance-id")
        .await?
        .text()
        .await?;

    for (_pid, proc_) in sys.processes().iter().filter(|(_, proc_)| proc_.name().to_lowercase().contains(filter_string)) {
        let proc_status = format!("{:?}", proc_.status());

        let client = CloudWatchClient::new(Region::region); // choose your AWS region

        let metric_value = if proc_status == "Run" { 0.0 } else { 1.0 };

        let dimension1 = Dimension {
            name: "ProcessStatus".to_string(),
            value: proc_status.to_string(),
        };

        let dimension2 = Dimension { 
            name: "instance-id".to_string(), 
            value: instance_id.to_string(), 
        };

        let datum = MetricDatum {
            dimensions: Some(vec![dimension1, dimension2]),
            metric_name: filter_string.to_string(),
            value: Some(metric_value),
            ..Default::default()
        };

        let put_metric_data_req = PutMetricDataInput {
            namespace: "Rust".to_string(),
            metric_data: vec![datum],
        };

        println!("Sending the following data to AWS CloudWatch: {:?}", put_metric_data_req);

        let mut retries = 0;
        loop {
            // Check if the total time exceeds 30 minutes, if yes break the loop
            if start.elapsed().as_secs() > 30 * 60 {
                break;
            }

            // Add timeout
            let send_result = timeout(Duration::from_secs(10), client.put_metric_data(put_metric_data_req.clone())).await;

            match send_result {
                Ok(result) => match result {
                    Ok(_) => {
                        println!("Metric sent to CloudWatch successfully.");
                        break; // success, break the loop
                    },
                    Err(e) => {
                        eprintln!("Error when sending metric to CloudWatch: {:?}", e);
                        retries += 1;
                        if retries >= 3 { // if number of retries exceed 3, break the loop
                            eprintln!("Failed to send metric after {} retries", retries);
                            break;
                        }
                    },
                },
                Err(e) => {
                    eprintln!("Timeout when sending metric to CloudWatch: {:?}", e);
                    retries += 1;
                    if retries >= 3 { // if number of retries exceed 3, break the loop
                        eprintln!("Failed to send metric after {} retries", retries);
                        break;
                    }
                },
            }

            // sleep for 1 second
            sleep(Duration::from_secs(180)).await;
        }
    }

    Ok(())
}

fn main() {
    // let sys = System::new_all();
    // let disks = parse_disk_data(&sys);
    // for disk in disks {
    //     let used_percentage = (disk.used as f64 / disk.total as f64) * 100.0;
    //     println!(
    //         "{} mounted on {}: {:.2}% used",
    //        disk.name, disk.mount_point, used_percentage
    //     );
    //     //println!("{:?}", disk);
    // }
    //let filter_string = "chrome".to_lowercase();

    //let rt = Runtime::new().unwrap();



    // rt.block_on(send_metrics_with_retries(&filter_string));

    // let filter_string = "chrome".to_lowercase();
        let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Please provide a configuration file.");
        std::process::exit(1);
    }

    let config_file = &args[1];

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        match read_config(config_file) {
            Ok(config) => {
                loop {
                let region = &config.region;
                for filter_string in &config.process_checks {
                    let filter_string = filter_string.to_lowercase();

                    
                        match send_metrics_with_retries(&filter_string, &region).await {
                            Ok(_) => println!("Metrics for {} sent successfully", filter_string),
                            Err(e) => eprintln!("Error occurred: {}", e),
                        }
                        
                    }
                    tokio::time::sleep(Duration::from_secs(30 * 60)).await; // Sleep for 30 minutes
                }
            },
            Err(e) => eprintln!("Error reading config file: {}", e),
        }
    });

} // end of main