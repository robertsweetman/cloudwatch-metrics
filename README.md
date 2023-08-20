Reads config file and send their status directly to Amazon Cloudwatch.

Relies on the fact that EC2 instances hold metadata about themselves, including credentials in their Environment to be able to post to Cloudwatch.

## ToDo

### Bugs
1. Fix release.yml pipeline as it's currently broken
2. Replace rusoto crate usage with aws-sdk-rust instead

### General

1. GitHub action to push application to S3
3. Windows client support
   - Is this basically a build option?
4. Figure out how to run scripts which are getting state
   - Include a test for this
5. More tests for the Rust code

### Features
1. Allow multiple config files?
### Ansible
1. Add a role to deploy the binary
2. Init scripts and systemd to start/stop service and set it to start on boot
3. Roles to deploy the config file live with each instance type install role i.e. webserver, dbserver etc.
