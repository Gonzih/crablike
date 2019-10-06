build:
	cargo build --release
run:
	nix-shell shell.nix --run 'cargo run'
rust-setup:
	rustup default nightly
ci:
	nix-shell shell.nix --run 'make build'
docker-image:
	docker build -t rust-build $(shell pwd)
ci-via-docker: docker-image
	docker run -v $(shell pwd):/code -t rust-build make build
