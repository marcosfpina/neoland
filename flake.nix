{
  description = "LlamaChat PoC Dev Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
            protobuf # Necessário para gRPC/Prost
          ];
          
          buildInputs = with pkgs; [
            rustToolchain
            gtk4
            libadwaita
            glib
            openssl
          ];
          
          # Garante que o protoc seja encontrado
          PROTOC = "${pkgs.protobuf}/bin/protoc";
        };
      }
    );
}
