with import <nixpkgs> {};
pkgs.mkShell {
  buildInputs = [
    glib pkgconfig SDL2
    rustup gnumake
  ];
}
