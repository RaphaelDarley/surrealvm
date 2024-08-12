# SurrealVM
## The Surreal Version Manager

### Install
```sh
cargo install surrealvm && surrealvm setup
```

### usage
```sh
# install surrealdb version (default latest)
surrealvm install latest
surrealvm install alpha
surrealvm install beta
surrealvm install nightly
surrealvm install 1.5.3

# set surreal to a specific version
surrealvm use latest
...
surrealvm use 1.5.3

# see all installed versions and which one is in use
surrealvm list

# remove surrealvm
surrealvm clean
```
