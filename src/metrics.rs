use sysinfo::{ProcessExt, System, SystemExt};
use rusoto_core::Region;
use rusoto_cloudwatch::{CloudWatch, CloudWatchClient, Dimension, MetricDatum, PutMetricDataInput};
use tokio::time::{timeout, Duration, sleep};
use std::time::Instant;

pub async fn send_metrics_with_retries(filter_string: &str, region: &str) -> Result<(), Box<dyn std::error::Error>> {
    let sys = System::new_all();

    let start = Instant::now();

    let instance_id = reqwest::get("http://169.254.169.254/latest/meta-data/instance-id")
        .await?
        .text()
        .await?;

    for (_pid, proc_) in sys.processes().iter().filter(|(_, proc_)| proc_.name().to_lowercase().contains(filter_string)) {
        let proc_status = format!("{:?}", proc_.status());

        let client = CloudWatchClient::new(Region::Custom {
            name: region.to_string(),
            endpoint: format!("https://monitoring.{}.amazonaws.com", region)
        }); // choose your AWS region

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