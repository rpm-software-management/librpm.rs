# Test Data Files

Offline RPM database snapshots extracted from container images. These are used
by the integration tests to verify that librpm can read real-world RPM databases
without requiring a full system install.

## Regenerating test databases

To regenerate or add new test databases, create a container from the target
image, copy its `/var/lib/rpm` directory, then remove the container. The only
file needed is `rpmdb.sqlite` (or the BDB files for older distros); lock files,
`-shm`, and `-wal` files can be deleted.

```bash
# CentOS Stream 9
id=$(podman create quay.io/centos/centos:stream9)
podman cp $id:/var/lib/rpm centos-stream-9
podman rm -v $id

# CentOS Stream 10
id=$(podman create quay.io/centos/centos:stream10)
podman cp $id:/var/lib/rpm centos-stream-10
podman rm -v $id

# Fedora 44
id=$(podman create registry.fedoraproject.org/fedora:44)
podman cp $id:/var/lib/rpm fedora-44
podman rm -v $id
```

After copying, remove unnecessary files from each directory:

```bash
rm -f */rpmdb.sqlite-shm */rpmdb.sqlite-wal */.rpm.lock */.rpmdbdirsymlink_created
```

Then update the corresponding test file in `tests/` with the correct package
count and first-package metadata. You can query these from the container:

```bash
# Package count
podman run --rm <image> rpm -qa | wc -l

# First package details (alphabetically)
podman run --rm <image> rpm -qa --queryformat '%{NAME}\n' | sort | head -1
podman run --rm <image> rpm -q <package> --queryformat \
  'NAME: %{NAME}\nEPOCH: %{EPOCH}\nVERSION: %{VERSION}\nRELEASE: %{RELEASE}\nARCH: %{ARCH}\nLICENSE: %{LICENSE}\nSUMMARY: %{SUMMARY}\n---\n%{DESCRIPTION}\n'
```
