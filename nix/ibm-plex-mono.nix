# IBM Plex Mono — Nix derivation for local-first font bundling.
#
# Downloads the font files from Google Fonts and exposes them as a
# Nix store path.  The Web Console CSS references these directly,
# avoiding any external CDN dependency.
#
# Usage:
#   nix build .#ibm-plex-mono
#   ls result/share/fonts/

{
  stdenvNoCC,
  lib,
  fetchzip,
}:

stdenvNoCC.mkDerivation rec {
  pname = "ibm-plex-mono";
  version = "2.3.0";

  src = fetchzip {
    url =
      "https://github.com/IBM/plex/releases/download/v${version}/TrueType.zip";
    hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="; # placeholder — fill after first build
  };

  installPhase = ''
    mkdir -p $out/share/fonts/truetype
    mkdir -p $out/share/fonts/woff2

    # Copy only the Mono weights we use (regular, bold, italic)
    for style in Regular Bold Italic BoldItalic; do
      cp IBM-Plex-Mono/fonts/ttf/IBMPlexMono-$style.ttf $out/share/fonts/truetype/ 2>/dev/null || true
    done
    cp IBM-Plex-Mono/fonts/ttf/IBMPlexMono-*.ttf $out/share/fonts/truetype/ 2>/dev/null || true

    # Copy woff2 if available (for web)
    cp IBM-Plex-Mono/fonts/woff2/IBMPlexMono-*.woff2 $out/share/fonts/woff2/ 2>/dev/null || true
  '';

  meta = {
    description = "IBM Plex Mono — the typeface for Neoland's terminal aesthetic";
    homepage = "https://github.com/IBM/plex";
    license = with lib.licenses; [ ofl ];
    platforms = lib.platforms.all;
    maintainers = [ "kernelcore" ];
  };
}
