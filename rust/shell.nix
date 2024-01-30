{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "shell-rust-ffmpeg";
  nativeBuildInputs = with pkgs; [ rustc cargo pkg-config ];
  buildInputs = with pkgs; [ ffmpeg_6-full libclang.lib ];
  LD_LIBRARY_PATH = "${pkgs.ffmpeg_6-full.lib}/lib";
  LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
}
