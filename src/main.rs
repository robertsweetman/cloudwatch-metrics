use sysinfo::{ProcessExt, DiskExt, System, SystemExt};
use rusoto_core::{Region, RusotoError};
use rusoto_cloudwatch::{CloudWatch, CloudWatchClient, Dimension, MetricDatum, PutMetricDataInput, PutMetricDataError};

#[derive(Debug)]
struct Disk {
    name: String,
    mount_point: String,
    // fs: Vec<char>,
    // disk_type: String,
    // removable: bool,
    used: u64,
    total: u64,
}

fn parse_disk_data(sys: &System) -> Vec<Disk> {
    sys.disks().iter().map(|disk_info| {
        let name = disk_info.name().to_str().unwrap().to_string();
        let mount_point = disk_info.mount_point().to_string_lossy().into_owned();
        // let mount_point = disk_info.mount_point().to_str().unwrap().to_string();

        // These values are placeholders as sysinfo doesn't provide them
        // let fs = vec!['a', 'p', 'f', 's'];
        // let disk_type = "SSD".to_string();
        // let removable = false;

        let used = disk_info.total_space() - disk_info.available_space();
        let total = disk_info.total_space();

        Disk {
            name,
            mount_point,
            // fs,
            // disk_type,
            // removable,
            used,
            total,
        }
    }).collect()
}


// fn get_disk_data() {
//     let mut sys = System::new_all();

//     // We refresh all information of our system
//     sys.refresh_all();

//     for disk in sys.disks() {
//         println!("{:?}", disk);
//     }
// }

async fn send_metric_to_cloudwatch(proc_status: &str, namespace: &str, metric_name: &str) -> Result<(), RusotoError<PutMetricDataError>> {
    let client = CloudWatchClient::new(Region::EuWest2); // choose your AWS region

    let metric_value = if proc_status == "Run" { 0.0 } else { 1.0 };

    let dimension = Dimension {
        name: "ProcessStatus".to_string(),
        value: proc_status.to_string(),
    };

    let datum = MetricDatum {
        dimensions: Some(vec![dimension]),
        metric_name: metric_name.to_string(),
        value: Some(metric_value),
        ..Default::default()
    };

    let put_metric_data_req = PutMetricDataInput {
        namespace: namespace.to_string(),
        metric_data: vec![datum],
    };

    match client.put_metric_data(put_metric_data_req).await {
        Ok(_) => println!("Metric sent to CloudWatch successfully."),
        Err(e) => eprintln!("Failed to send metric to CloudWatch: {}", e),
    }

    Ok(())
}

fn main() {
    let sys = System::new_all();
    let disks = parse_disk_data(&sys);
    for disk in disks {
        let used_percentage = (disk.used as f64 / disk.total as f64) * 100.0;
        println!(
            "{} mounted on {}: {:.2}% used",
           disk.name, disk.mount_point, used_percentage
        );
        //println!("{:?}", disk);
    }
    let filter_string = "chrome".to_lowercase();

    for (pid, proc_) in sys.processes().iter().filter(|(_,proc_)| proc_.name().to_lowercase().contains(&filter_string)) {
        println!("{}:{} => status: {:?}", pid, proc_.name(), proc_.status());
        let proc_status = format!("{:?}", proc_.status());
    let _ = send_metric_to_cloudwatch(&proc_status, "Rust", &filter_string);
    }
}