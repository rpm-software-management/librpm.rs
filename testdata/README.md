# Test Data Files

## Create test files from the command line

Within this directory, run the following commands to create new test files:

```bash
# Fetching the centos:7 rpm databases
id=$(docker create centos:7)
docker cp $id:/var/lib/rpm centos7
docker rm -v $id
```