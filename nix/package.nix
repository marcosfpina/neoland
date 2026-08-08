{ pkgs
, src ? ../.
, version ? (builtins.fromTOML (builtins.readFile (src + "/Cargo.toml"))).package.version
}:

let
  # utoipa-swagger-ui baixa este zip em build time; pré-buscar mantém
  # o build determinístico e sem rede no builder.
  swaggerUiZip = pkgs.fetchurl {
    url = "https://github.com/swagger-api/swagger-ui/archive/refs/tags/v5.17.12.zip";
    sha256 = "1wh7yrkyc87zkzdyxbhpzcvykmmz2zkbsj9h0ny2mmryjby37bhw";
  };
in
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

  # O zip precisa ser gravável: o build script copia preservando perms
  # e o unzip falha com PermissionDenied se vier 444 do /nix/store.
  preBuild = ''
    cp ${swaggerUiZip} "$TMPDIR/v5.17.12.zip"
    chmod +w "$TMPDIR/v5.17.12.zip"
    export SWAGGER_UI_DOWNLOAD_URL="file://$TMPDIR/v5.17.12.zip"
  '';

  # Sandbox sem rede/DB: só testes de unidade; embeddings desabilitados.
  doCheck = true;
  cargoTestFlags = [ "--lib" ];
  NEOLAND_SKIP_EMBEDDINGS = "true";

  meta = with pkgs.lib; {
    description = "Neoland";
    license = licenses.mit;
    maintainers = [ "marcosfpina" ];
    mainProgram = "neoland";
  };
}
