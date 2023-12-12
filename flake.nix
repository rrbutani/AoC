{
  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs;
    flu.url = github:numtide/flake-utils;
    rust-overlay.url = github:oxalica/rust-overlay;
  };

  outputs = {
    self, nixpkgs, flu, rust-overlay
  }: flu.lib.eachDefaultSystem (system:
    let
      # We need this because tools like `XCode Instruments` rely on the UUID to
      # correlate debug info files with traces.
      mkStdenv = pkgs:
        if pkgs.targetPlatform.isDarwin then
          let
            cc = pkgs.stdenv.cc;
            bintools = cc.bintools.overrideAttrs (old: {
              postFixup = old.postFixup + ''
                sed -i 's/-no_uuid//g' $out/nix-support/libc-ldflags-before
              '';
            });
          in
          pkgs.stdenv.override {
            cc = cc.override { inherit bintools; };
            allowedRequisites = null;
          }
        else
          pkgs.stdenv;

      # Bad hack; we want to override `stdenv` but only in this overlay...
      rust-overlay' = final: prev: let
        overlay = import rust-overlay;
        stdenv = mkStdenv final;
        callPackage = path: overrides: final.callPackage path (overrides // { inherit stdenv; });
      in overlay (final // { inherit callPackage; }) prev;

      np = import nixpkgs {
        overlays = [ rust-overlay' ];
        inherit system;
      };
      lib = np.lib;

      stdenv = mkStdenv np;

      toolchain = np.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

      buildInputs = lib.optionals np.targetPlatform.isDarwin
        (with np.darwin.apple_sdk.frameworks; [
          Security
        ]);
    in
    {
      packages = {
        inherit stdenv toolchain;
        inherit (stdenv) cc;
        inherit (stdenv.cc) bintools;

        inherit np;
      };
      devShells.default = (np.mkShell.override { inherit stdenv; }) {
        # inherit buildInputs;

        nativeBuildInputs = with np; [
          toolchain
          python311
          bashInteractive

          cargo-expand

          # TODO: gate on Linux:
          pkg-config
        ];
        buildInputs = buildInputs ++ (with np; [
          # TODO: gate on Linux:
          openssl

          fontconfig
        ]);
      };
    });
}
