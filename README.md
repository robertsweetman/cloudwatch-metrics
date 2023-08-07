Reads config file and send their status directly to Amazon Cloudwatch.

Relies on the fact that EC2 instances hold metadata about themselves, including credentials in their Environment to be able to post to Cloudwatch.

## ToDo

### General
1. GitHub action to build 
2. GitHub action to push application to S3
3. FIXME: fix release.yml pipeline as it's currently broken
2. Windows client support
3. Figure out how to handle other collectd scripts which are getting state
7. Tests for the Rust code
8. Multiple config files?

### Ansible
1. Add a role to deploy the binary
2. Init scripts for Rhel 6 and systemd for Rhel 7 to start/stop service and set it to start on boot
3. Roles to deploy the config file live with each instance type install role i.e. nomis-db, nomis-web
