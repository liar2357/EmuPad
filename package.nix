{
  lib,
  rustPlatform,
  stdenv,
}:

rustPlatform.buildRustPackage {
  pname = "emu-pad";
  version = "0.1.0";

  src = ./.;

  cargoLock.lockFile = ./Cargo.lock;

  installPhase = ''
    runHook preInstall

    install -Dm755 \
      target/${stdenv.hostPlatform.rust.rustcTarget}/release/emu-pad \
      $out/bin/emu-pad

    install -Dm755 \
      target/${stdenv.hostPlatform.rust.rustcTarget}/release/emu-padctl \
      $out/bin/emu-padctl

    runHook postInstall
  '';

  meta = {
    description = "Virtual trackpad for Linux with touchscreen";
    homepage = "https://github.com/liar2357/emu-pad";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
    mainProgram = "emu-pad";
  };
}
