with import <nixpkgs>{};
mkShell {
  NIX_LD_LIBRARY_PATH = lib.makeLibraryPath [
    stdenv.cc.cc
    xorg.libX11
    xorg.libXcursor
    xorg.libXinerama
    xorg.libXrandr
    xorg.libXi
    libpulseaudio
    alsaLib
    libGL
    xorg.libpthreadstubs
    glibc
  ];
  NIX_LD = builtins.readFile "${stdenv.cc}/nix-support/dynamic-linker";
  shellHook = ''
    ./Wonderdraft.x86_64
  '';

}
