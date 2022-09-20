{ naersk, src, lib, pkg-config, cmake, zlib, openssl }:

naersk.buildPackage {
  pname = "barrel";
  version = "0.1.0";

  src = ./.;

  cargoSha256 = lib.fakeSha256;

  nativeBuildInputs = [ pkg-config cmake ];
  buildInputs = [ zlib openssl ];

  meta = with lib; {
    description = "Simple package manager for lingua franca";
    homepage = "https://github.com/revol-xut/barrel";
  };
}
