{ pkgs
, src ? ../.
, version ? "0.1.0"
}:

pkgs.rustPlatform.buildRustPackage {
  pname = "neoland";
  inherit version src;

  cargoLock = {
    lockFile = src + "/Cargo.lock";
    allowBuiltinFetchGit = true;
  };

  nativeBuildInputs = with pkgs; [
    pkg-config
    protobuf
    maturin
  ];

  buildInputs = with pkgs; [
    openssl
  ];

  PROTOC = "${pkgs.protobuf}/bin/protoc";

  doCheck = true;

  meta = with pkgs.lib; {
    description = "Neoland - Terminal AI Agent with Enterprise Security";
    license = licenses.mit;
    maintainers = [ "kernelcore" ];
    mainProgram = "neoland";
  };
}
