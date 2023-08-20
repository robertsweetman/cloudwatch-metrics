// use std::os::unix::process;
mod metrics;
mod scripts;

use cloudwatch_metrics::read_config;
use metrics::send_metrics_with_retries;
use scripts::{get_filename_from_path, run_script_and_get_status};

use tokio::runtime::Runtime;
use tokio::time::Duration;

fn main() {
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
                let polling_interval = &config.polling_interval_minutes;
                    for filter_string in &config.process_checks {
                        let filter_string = filter_string.to_lowercase();
                    
                            match send_metrics_with_retries(&filter_string, &region).await {
                                Ok(_) => println!("Metrics for {} sent successfully", filter_string),
                                Err(e) => eprintln!("Error occurred: {}", e),
                            }
                        
                    }
                    for item in &config.script_paths {

                        let script_name_option = get_filename_from_path(&item.path);    

                        match script_name_option {
                            Some(script_name) => println!("File name is {}", script_name),
                            None => println!("No file name found"),
                        }

                        let script_name = script_name_option.unwrap_or_else(|| "No script name found");

                            match run_script_and_get_status(&item).await {


                                Ok(status) => {
                                    println!("Script {} ran successfully with status code {}", script_name, status);

                                    match send_metrics_with_retries(
                                        &status.to_string(),
                                        &region,
                                    ).await {
                                        Ok(_) => println!("Metrics for {} sent successfully", script_name),
                                        Err(e) => eprintln!("Error occurred: {}", e),

                                    }
                                }
                                Err(e) => eprintln!("Error running script {}: {}", script_name, e),
                            }
                    }

                    tokio::time::sleep(Duration::from_secs(polling_interval * 60)).await; // Sleep for 30 minutes
                }
            },
            Err(e) => eprintln!("Error reading config file: {}", e),
        }
    });

} // end of main