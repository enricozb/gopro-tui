{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cargo
    clippy
    rustc
    exiftool
    mpv
    ffmpeg-full
  ];
}
