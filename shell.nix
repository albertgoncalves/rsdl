with import <nixpkgs> {};
let
    shared = [
        rustup
        SDL2
        shellcheck
    ];
    hook = ''
        . .shellhook
    '';
in
{
    darwin = mkShell {
        buildInputs = [
            (with darwin.apple_sdk.frameworks; [
                AppKit
                OpenGL
                Security
            ])
        ] ++ shared;
        shellHook = hook;
    };
    linux = mkShell {
        buildInputs = [
            pkg-config
        ] ++ shared;
        APPEND_LIBRARY_PATH = stdenv.lib.makeLibraryPath [
            libGL
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
        ];
        shellHook = ''
            export LD_LIBRARY_PATH="$APPEND_LIBRARY_PATH:$LD_LIBRARY_PATH"
            expression=$(grep "export" < nixGL/result/bin/nixGL*)
            if [ -n "$expression" ]; then
                eval "$expression"
            fi
        '' + hook;
    };
}
