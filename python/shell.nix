{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "shell-python";
  nativeBuildInputs = [
    (pkgs.python3.withPackages (ps: with ps; [
      pycryptodomex
    ]))
  ];
}
