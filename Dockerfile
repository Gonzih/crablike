FROM nixos/nix

MAINTAINER Max Gonzih <gonzih at gmail dot com>

RUN nix-channel --add https://nixos.org/channels/nixpkgs-unstable nixos
RUN nix-channel --update
RUN nix-env -i gnumake rustup
RUN rustup default nightly

WORKDIR /code
