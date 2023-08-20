use cloudwatch_metrics::read_config;

#[test]
fn test_read_config() {
    let config = read_config("test_config.yml").unwrap();
    assert_eq!(config.polling_interval_minutes, 60);
    assert_eq!(config.region, "EuWest2");
    assert_eq!(config.process_checks[0], "bash");
    assert_eq!(config.process_checks[1], "Slack");
}