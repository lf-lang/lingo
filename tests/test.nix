{pkgs, lib, source, stdenv, lingo, jdk17_headless, lingua-franca}:
stdenv.mkDerivation {
  name = "lingua-franca-tests-cpp";

  src = source;

  buildPhase = ''
    cp ${./Lingo.toml} ./test/Cpp/Lingo.toml
    cd test/Cpp
    ls ${lingua-franca}/bin
    ${lingo}/bin/lingo build --lfc ${lingua-franca}/bin/lfc --build-system c-make
  '';

  installPhase = ''
    mkdir -p $out/bin
    cp -r ./bin/* $out/bin/
  '';
}
