# Assets

## Create test files from the command line

Within the assets directory, run the following commands to create the test files:

```bash
# Fetching the centos:7 rpm databaes
id=$(docker create centos:7)
docker cp $id:/var/lib/rpm centos7
docker rm -v $id
```