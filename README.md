# Go links

Go links server written in Rust.

![Docker Pulls](https://img.shields.io/docker/pulls/prateekkumarweb/go?style=for-the-badge)
![Docker Image Size](https://img.shields.io/docker/image-size/prateekkumarweb/go/alpha?style=for-the-badge&arch=arm64)
![Docker Stars](https://img.shields.io/docker/stars/prateekkumarweb/go?style=for-the-badge)

## Build

```sh
$ cargo build --release
```

### Check unused deps

```sh
$ cargo +nightly udeps
```

### Docker build

```sh
$ docker buildx build --platform=linux/amd64,linux/arm64 . -t prateekkumarweb/go:alpha --push
```

## Usage

### With cargo

Run the below commands inside the repository.

```sh
# Initlize config file with username and password
$ cargo run --release -- init
# Start the server
$ cargo run --release -- --config path/to/config.yaml
```

### With docker

```sh
# Initlize config file with username and password
$ docker run --rm -it -v data:/app/data go:alpha init
# Start the server
$ docker run -dp 3000:3000 -e RUST_LOG=info -v data:/app/data go:alpha
```
